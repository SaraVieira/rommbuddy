use reqwest::Client;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseBackend,
    DatabaseConnection, EntityTrait, QueryFilter, Statement,
};
use serde::Deserialize;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use std::collections::HashMap;

use crate::dedup;
use crate::error::{AppError, AppResult};
use crate::models::{ConnectionTestResult, ScanProgress, TokenPair};
use crate::platform_registry;

/// ROMM API response types (deserialized from JSON).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RommTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RommPlatform {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub rom_count: i64,
    pub display_name: String,
    pub igdb_id: Option<i64>,
    pub moby_id: Option<i64>,
    #[serde(default)]
    pub is_unidentified: bool,
}

/// Paginated response wrapper from ROMM API.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RommPageResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RommRom {
    pub id: i64,
    pub igdb_id: Option<i64>,
    pub platform_id: i64,
    pub platform_slug: String,
    pub platform_display_name: String,
    pub fs_name: String,
    pub name: Option<String>,
    pub fs_size_bytes: Option<i64>,
    #[serde(default)]
    pub regions: Vec<String>,
    pub summary: Option<String>,
    pub url_cover: Option<String>,
    /// Nested metadata object.
    pub metadatum: Option<RommMetadatum>,
}

#[derive(Debug, Deserialize)]
pub struct RommMetadatum {
    #[serde(default)]
    pub genres: Vec<String>,
    pub first_release_date: Option<i64>,
}

pub struct RommClient {
    base_url: String,
    username: String,
    password: String,
    client: Client,
    tokens: RwLock<Option<TokenPair>>,
}

impl RommClient {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(base_url: String, username: String, password: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            username,
            password,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            tokens: RwLock::new(None),
        }
    }

    /// Authenticate with username/password.
    async fn authenticate(&self) -> AppResult<TokenPair> {
        let url = format!("{}/api/token", self.base_url);
        let resp = self
            .client
            .post(&url)
            .form(&[
                ("username", self.username.as_str()),
                ("password", self.password.as_str()),
                ("grant_type", "password"),
                ("scope", "me.read roms.read platforms.read assets.read"),
            ])
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Auth(format!(
                "Authentication failed ({status}): {body}"
            )));
        }

        let token_resp: RommTokenResponse = resp.json().await?;
        Ok(TokenPair {
            access_token: token_resp.access_token,
            refresh_token: token_resp.refresh_token,
        })
    }

    /// Get a valid access token, refreshing or re-authenticating if needed.
    async fn get_token(&self) -> AppResult<String> {
        {
            let tokens = self.tokens.read().await;
            if let Some(ref tp) = *tokens {
                return Ok(tp.access_token.clone());
            }
        }
        // No token, authenticate -- re-check after acquiring write lock
        let mut tokens = self.tokens.write().await;
        if let Some(ref tp) = *tokens {
            return Ok(tp.access_token.clone());
        }
        let tp = self.authenticate().await?;
        let access = tp.access_token.clone();
        *tokens = Some(tp);
        drop(tokens);
        Ok(access)
    }

    /// Make an authenticated GET request, retrying once on 401.
    async fn auth_get(&self, url: &str) -> AppResult<reqwest::Response> {
        let token = self.get_token().await?;
        let resp = self
            .client
            .get(url)
            .bearer_auth(&token)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            // Token expired, re-authenticate
            let tp = self.authenticate().await?;
            let new_token = tp.access_token.clone();
            *self.tokens.write().await = Some(tp);

            let resp = self
                .client
                .get(url)
                .bearer_auth(&new_token)
                .send()
                .await?;
            Ok(resp)
        } else {
            Ok(resp)
        }
    }

    /// Test connection: authenticate, count platforms and ROMs.
    pub async fn test_connection(&self) -> AppResult<ConnectionTestResult> {
        self.authenticate().await?;

        let platforms = self.get_platforms().await?;
        #[allow(clippy::cast_possible_truncation)]
        let platform_count = platforms.len() as u32;

        // Get total ROM count by summing platform rom_counts
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let rom_count: u32 = platforms.iter().map(|p| p.rom_count as u32).sum();

        Ok(ConnectionTestResult {
            platform_count,
            rom_count,
        })
    }

    /// Get all platforms from ROMM.
    pub async fn get_platforms(&self) -> AppResult<Vec<RommPlatform>> {
        let url = format!("{}/api/platforms", self.base_url);
        let resp = self.auth_get(&url).await?;
        if !resp.status().is_success() {
            return Err(AppError::Other(format!(
                "Failed to get platforms: {}",
                resp.status()
            )));
        }
        let platforms: Vec<RommPlatform> = resp.json().await?;
        Ok(platforms)
    }

    /// Get ROMs with pagination (all platforms).
    async fn get_roms_page(
        &self,
        limit: i64,
        offset: i64,
    ) -> AppResult<RommPageResponse<RommRom>> {
        let url = format!(
            "{}/api/roms?limit={limit}&offset={offset}",
            self.base_url,
        );
        let resp = self.auth_get(&url).await?;
        if !resp.status().is_success() {
            return Err(AppError::Other(format!(
                "Failed to get ROMs: {}",
                resp.status()
            )));
        }
        let page: RommPageResponse<RommRom> = resp.json().await.map_err(|e| {
            AppError::Other(format!("Failed to parse ROMs response: {e}"))
        })?;
        Ok(page)
    }

    /// Sync all ROMs from ROMM into local database.
    pub async fn sync_to_db(
        &self,
        source_id: i64,
        db: &DatabaseConnection,
        on_progress: impl Fn(ScanProgress) + Send,
        cancel: CancellationToken,
    ) -> AppResult<()> {
        let platforms = self.get_platforms().await?;
        #[allow(clippy::cast_sign_loss)]
        let total_roms: u64 = platforms.iter().map(|p| p.rom_count as u64).sum();
        let mut current: u64 = 0;

        // Phase 1: Build a map from ROMM platform ID -> local platform ID
        let mut platform_map: HashMap<i64, i64> = HashMap::new();

        for platform in &platforms {
            if cancel.is_cancelled() {
                return Ok(());
            }

            // Skip unidentified platforms (folders that aren't real systems)
            if platform.is_unidentified {
                log::info!(
                    "Skipping unidentified platform: '{}' ({})",
                    platform.slug,
                    platform.display_name
                );
                continue;
            }

            // Map ROMM platform to our canonical slug
            let canonical_slug = platform_registry::resolve_romm_slug(&platform.slug);

            // Find or create the platform in our DB
            use crate::entity::platforms;
            let existing = platforms::Entity::find()
                .filter(platforms::Column::Slug.eq(&canonical_slug))
                .one(db)
                .await?;
            let local_platform_id = if let Some(p) = existing {
                p.id
            } else {
                // Auto-create the platform
                log::info!(
                    "Creating new platform: slug='{canonical_slug}', name='{}'",
                    platform.display_name
                );
                let model = platforms::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    slug: Set(canonical_slug.clone()),
                    name: Set(platform.display_name.clone()),
                    igdb_id: Set(None),
                    screenscraper_id: Set(platform_registry::ss_id(&canonical_slug).map(|id| id as i64)),
                    file_extensions: Set("[]".to_string()),
                    folder_aliases: Set("[]".to_string()),
                    created_at: Set(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
                    updated_at: Set(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
                }.insert(db).await?;
                model.id
            };

            platform_map.insert(platform.id, local_platform_id);
        }

        // Phase 2: Fetch ALL ROMs in one paginated pass, using the map to assign platforms
        let page_size = 50i64;
        let mut offset = 0i64;
        loop {
            if cancel.is_cancelled() {
                return Ok(());
            }

            let page = self.get_roms_page(page_size, offset).await?;
            if page.items.is_empty() {
                break;
            }

            for rom in &page.items {
                if cancel.is_cancelled() {
                    return Ok(());
                }

                // Look up local platform ID from the map using the ROM's own platform_id
                let Some(&local_platform_id) = platform_map.get(&rom.platform_id) else {
                    // ROM belongs to an unidentified or unknown platform -- skip it
                    current += 1;
                    continue;
                };

                current += 1;
                let rom_name = rom.name.clone().unwrap_or_else(|| rom.fs_name.clone());
                on_progress(ScanProgress {
                    source_id,
                    total: total_roms,
                    current,
                    current_item: rom_name.clone(),
                });

                // Upsert into roms + source_roms via dedup logic
                let regions_json =
                    serde_json::to_string(&rom.regions).unwrap_or_else(|_| "[]".to_string());
                let source_rom_id_str = rom.id.to_string();
                let source_url = format!(
                    "{}/api/roms/{}/content/{}",
                    self.base_url, rom.id, rom.fs_name
                );
                let rom_id = dedup::upsert_rom_deduped(
                    db,
                    local_platform_id,
                    &rom_name,
                    &rom.fs_name,
                    rom.fs_size_bytes,
                    &regions_json,
                    None,
                    source_id,
                    Some(&source_rom_id_str),
                    Some(&source_url),
                )
                .await?;

                // Upsert metadata
                let genres: Vec<String> = rom
                    .metadatum
                    .as_ref()
                    .map(|m| m.genres.clone())
                    .unwrap_or_default();
                let genres_json =
                    serde_json::to_string(&genres).unwrap_or_else(|_| "[]".to_string());
                let release_date = rom.metadatum.as_ref().and_then(|m| {
                    m.first_release_date.and_then(|ts| {
                        // Detect milliseconds vs seconds: if ts > year 3000 in seconds, divide by 1000
                        let ts = if ts > 32_503_680_000 { ts / 1000 } else { ts };
                        chrono::DateTime::from_timestamp(ts, 0)
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                    })
                });

                db.execute(Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    "INSERT INTO metadata (rom_id, description, genres, release_date)
                     VALUES (?, ?, ?, ?)
                     ON CONFLICT(rom_id) DO UPDATE SET
                       description = COALESCE(excluded.description, metadata.description),
                       genres = excluded.genres,
                       release_date = COALESCE(excluded.release_date, metadata.release_date),
                       updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
                    [rom_id.into(), rom.summary.clone().into(), genres_json.into(), release_date.into()],
                ))
                .await?;

                // Upsert cover artwork
                if let Some(ref cover_url) = rom.url_cover {
                    let full_url = if cover_url.starts_with("http") {
                        cover_url.clone()
                    } else {
                        format!("{}{cover_url}", self.base_url)
                    };
                    db.execute(Statement::from_sql_and_values(
                        DatabaseBackend::Sqlite,
                        "INSERT INTO artwork (rom_id, art_type, url) VALUES (?, 'cover', ?) ON CONFLICT(rom_id, art_type, url) DO NOTHING",
                        [rom_id.into(), full_url.clone().into()],
                    ))
                    .await?;
                }
            }

            offset += page_size;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            if page.items.len() < page_size as usize {
                break;
            }
        }

        // Update source last_synced_at
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "UPDATE sources SET last_synced_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?",
            [source_id.into()],
        )).await?;

        Ok(())
    }

    /// Download a ROM file by its ROMM ID and `file_name`, returning bytes.
    pub async fn download_rom(
        &self,
        romm_rom_id: i64,
        file_name: &str,
    ) -> AppResult<reqwest::Response> {
        let encoded_name = urlencoding::encode(file_name);
        let url = format!(
            "{}/api/roms/{romm_rom_id}/content/{encoded_name}",
            self.base_url,
        );
        let resp = self.auth_get(&url).await?;
        if !resp.status().is_success() {
            return Err(AppError::Other(format!(
                "Failed to download ROM: {}",
                resp.status()
            )));
        }
        Ok(resp)
    }

    /// Proxy an image URL, returning base64-encoded data URL string.
    pub async fn proxy_image(&self, url: &str) -> AppResult<String> {
        use base64::Engine;
        let resp = if url.contains("/api/") {
            // Authenticated ROMM endpoint
            self.auth_get(url).await?
        } else {
            // Public asset URL
            self.client.get(url).send().await?
        };
        if !resp.status().is_success() {
            return Err(AppError::Other(format!(
                "Failed to proxy image: {}",
                resp.status()
            )));
        }
        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/jpeg")
            .to_string();
        const MAX_IMAGE_SIZE: u64 = 10 * 1024 * 1024; // 10 MB
        if let Some(len) = resp.content_length() {
            if len > MAX_IMAGE_SIZE {
                return Err(AppError::Other(format!(
                    "Image too large: {len} bytes (max {MAX_IMAGE_SIZE})"
                )));
            }
        }
        let bytes = resp.bytes().await?;
        if bytes.len() as u64 > MAX_IMAGE_SIZE {
            return Err(AppError::Other(format!(
                "Image too large: {} bytes (max {MAX_IMAGE_SIZE})",
                bytes.len()
            )));
        }
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        Ok(format!("data:{content_type};base64,{b64}"))
    }
}
