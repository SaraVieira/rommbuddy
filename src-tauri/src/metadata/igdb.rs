use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, Semaphore};

use crate::error::{AppError, AppResult};

// ---------------------------------------------------------------------------
// IGDB response types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgdbGameData {
    pub id: i64,
    pub name: Option<String>,
    pub summary: Option<String>,
    pub storyline: Option<String>,
    pub aggregated_rating: Option<f64>,
    pub first_release_date: Option<i64>,
    pub genres: Option<Vec<IgdbNamedItem>>,
    pub themes: Option<Vec<IgdbNamedItem>>,
    pub game_modes: Option<Vec<IgdbNamedItem>>,
    pub player_perspectives: Option<Vec<IgdbNamedItem>>,
    pub cover: Option<IgdbImage>,
    pub screenshots: Option<Vec<IgdbImage>>,
    pub involved_companies: Option<Vec<IgdbInvolvedCompany>>,
    pub franchises: Option<Vec<IgdbNamedItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgdbNamedItem {
    pub id: Option<i64>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgdbImage {
    pub id: Option<i64>,
    pub image_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgdbInvolvedCompany {
    pub company: Option<IgdbCompany>,
    pub developer: Option<bool>,
    pub publisher: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgdbCompany {
    pub id: Option<i64>,
    pub name: Option<String>,
}

// ---------------------------------------------------------------------------
// Token management
// ---------------------------------------------------------------------------

struct TokenState {
    access_token: String,
    expires_at: Instant,
}

// ---------------------------------------------------------------------------
// IgdbClient
// ---------------------------------------------------------------------------

pub struct IgdbClient {
    client_id: String,
    client_secret: String,
    http: reqwest::Client,
    token: Arc<RwLock<Option<TokenState>>>,
    semaphore: Arc<Semaphore>,
    last_request: Arc<RwLock<Instant>>,
}

impl IgdbClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            http: reqwest::Client::builder()
                .user_agent("romm-buddy/0.1")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            token: Arc::new(RwLock::new(None)),
            semaphore: Arc::new(Semaphore::new(4)),
            last_request: Arc::new(RwLock::new(Instant::now() - std::time::Duration::from_secs(1))),
        }
    }

    /// Ensure we have a valid OAuth2 token, refreshing if needed.
    async fn ensure_token(&self) -> AppResult<String> {
        // Check if current token is still valid
        {
            let guard = self.token.read().await;
            if let Some(ref state) = *guard {
                if Instant::now() < state.expires_at {
                    return Ok(state.access_token.clone());
                }
            }
        }

        // Acquire write lock and fetch new token
        let mut guard = self.token.write().await;
        // Double-check after acquiring write lock
        if let Some(ref state) = *guard {
            if Instant::now() < state.expires_at {
                return Ok(state.access_token.clone());
            }
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            expires_in: u64,
        }

        let resp = self
            .http
            .post("https://id.twitch.tv/oauth2/token")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("grant_type", "client_credentials"),
            ])
            .send()
            .await
            .map_err(|e| AppError::Other(format!("IGDB token request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Other(format!(
                "IGDB token request returned {status}: {body}"
            )));
        }

        let token_resp: TokenResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Other(format!("Failed to parse IGDB token response: {e}")))?;

        let expires_at = Instant::now()
            + std::time::Duration::from_secs(token_resp.expires_in.saturating_sub(60));

        let access_token = token_resp.access_token.clone();
        *guard = Some(TokenState {
            access_token: token_resp.access_token,
            expires_at,
        });

        Ok(access_token)
    }

    /// Rate-limited query to IGDB API.
    async fn query(&self, endpoint: &str, body: &str) -> AppResult<String> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| AppError::Other(format!("Semaphore error: {e}")))?;

        // Enforce minimum 250ms between requests (single write lock to prevent races)
        {
            let mut last = self.last_request.write().await;
            let elapsed = last.elapsed();
            let min_interval = std::time::Duration::from_millis(250);
            if elapsed < min_interval {
                tokio::time::sleep(min_interval - elapsed).await;
            }
            *last = Instant::now();
        }

        let token = self.ensure_token().await?;
        let url = format!("https://api.igdb.com/v4/{endpoint}");

        let resp = self
            .http
            .post(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "text/plain")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| AppError::Other(format!("IGDB API request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Other(format!(
                "IGDB API returned {status}: {body}"
            )));
        }

        resp.text()
            .await
            .map_err(|e| AppError::Other(format!("Failed to read IGDB response: {e}")))
    }

    /// Fetch multiple games by their IGDB IDs (batch query).
    pub async fn fetch_games_by_ids(&self, ids: &[i64]) -> AppResult<Vec<IgdbGameData>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let id_list: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let body = format!(
            "fields name, summary, storyline, aggregated_rating, first_release_date, \
             genres.name, themes.name, game_modes.name, player_perspectives.name, \
             cover.image_id, screenshots.image_id, \
             involved_companies.company.name, involved_companies.developer, involved_companies.publisher, \
             franchises.name; \
             where id = ({}); \
             limit {};",
            id_list.join(","),
            ids.len()
        );

        let response = self.query("games", &body).await?;
        let games: Vec<IgdbGameData> = serde_json::from_str(&response)
            .map_err(|e| AppError::Other(format!("Failed to parse IGDB games response: {e}")))?;

        Ok(games)
    }

    /// Search for a game by name.
    pub async fn search_game(&self, name: &str) -> AppResult<Option<IgdbGameData>> {
        let escaped = name.replace('"', "\\\"");
        let body = format!(
            "fields name, summary, storyline, aggregated_rating, first_release_date, \
             genres.name, themes.name, game_modes.name, player_perspectives.name, \
             cover.image_id, screenshots.image_id, \
             involved_companies.company.name, involved_companies.developer, involved_companies.publisher, \
             franchises.name; \
             search \"{escaped}\"; \
             limit 1;"
        );

        let response = self.query("games", &body).await?;
        let games: Vec<IgdbGameData> = serde_json::from_str(&response)
            .map_err(|e| AppError::Other(format!("Failed to parse IGDB search response: {e}")))?;

        Ok(games.into_iter().next())
    }

    /// Test connection by attempting token acquisition.
    pub async fn test_connection(&self) -> AppResult<crate::models::IgdbTestResult> {
        match self.ensure_token().await {
            Ok(_) => Ok(crate::models::IgdbTestResult {
                success: true,
                message: "Successfully authenticated with IGDB/Twitch".to_string(),
            }),
            Err(e) => Ok(crate::models::IgdbTestResult {
                success: false,
                message: format!("Authentication failed: {e}"),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Helper functions for extracting data from IGDB responses
// ---------------------------------------------------------------------------

impl IgdbGameData {
    pub fn developer(&self) -> Option<String> {
        self.involved_companies.as_ref().and_then(|companies| {
            companies
                .iter()
                .find(|c| c.developer == Some(true))
                .and_then(|c| c.company.as_ref())
                .and_then(|c| c.name.clone())
        })
    }

    pub fn publisher(&self) -> Option<String> {
        self.involved_companies.as_ref().and_then(|companies| {
            companies
                .iter()
                .find(|c| c.publisher == Some(true))
                .and_then(|c| c.company.as_ref())
                .and_then(|c| c.name.clone())
        })
    }

    pub fn genre_names(&self) -> Vec<String> {
        self.genres
            .as_ref()
            .map(|g| g.iter().filter_map(|i| i.name.clone()).collect())
            .unwrap_or_default()
    }

    pub fn theme_names(&self) -> Vec<String> {
        self.themes
            .as_ref()
            .map(|t| t.iter().filter_map(|i| i.name.clone()).collect())
            .unwrap_or_default()
    }

    pub fn game_mode_names(&self) -> Vec<String> {
        self.game_modes
            .as_ref()
            .map(|m| m.iter().filter_map(|i| i.name.clone()).collect())
            .unwrap_or_default()
    }

    pub fn player_perspective_names(&self) -> Vec<String> {
        self.player_perspectives
            .as_ref()
            .map(|p| p.iter().filter_map(|i| i.name.clone()).collect())
            .unwrap_or_default()
    }

    pub fn cover_image_id(&self) -> Option<String> {
        self.cover.as_ref().and_then(|c| c.image_id.clone())
    }

    pub fn screenshot_image_ids(&self) -> Vec<String> {
        self.screenshots
            .as_ref()
            .map(|s| s.iter().filter_map(|i| i.image_id.clone()).collect())
            .unwrap_or_default()
    }

    pub fn franchise_name(&self) -> Option<String> {
        self.franchises
            .as_ref()
            .and_then(|f| f.first())
            .and_then(|f| f.name.clone())
    }

    pub fn cover_url(&self) -> Option<String> {
        self.cover_image_id()
            .map(|id| format!("https://images.igdb.com/igdb/image/upload/t_cover_big/{id}.jpg"))
    }

    pub fn screenshot_urls(&self) -> Vec<String> {
        self.screenshot_image_ids()
            .into_iter()
            .map(|id| {
                format!("https://images.igdb.com/igdb/image/upload/t_screenshot_big/{id}.jpg")
            })
            .collect()
    }

    pub fn first_release_date_string(&self) -> Option<String> {
        self.first_release_date.map(|ts| {
            chrono::DateTime::from_timestamp(ts, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_default()
        })
    }

    /// Build the description from summary + storyline.
    pub fn description(&self) -> Option<String> {
        match (&self.summary, &self.storyline) {
            (Some(s), Some(st)) if !st.is_empty() => Some(format!("{s}\n\n{st}")),
            (Some(s), _) => Some(s.clone()),
            (None, Some(st)) if !st.is_empty() => Some(st.clone()),
            _ => None,
        }
    }
}
