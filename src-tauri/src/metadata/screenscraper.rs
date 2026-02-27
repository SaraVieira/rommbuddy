use reqwest::Client;
use sqlx::SqlitePool;
use std::time::Duration;

use crate::error::{AppError, AppResult};

// ---------------------------------------------------------------------------
// Developer credentials (identify this app to ScreenScraper)
// ---------------------------------------------------------------------------

const DEV_ID: &str = "NikkitaFTW";
const DEV_PASSWORD: &str = "5RnA96uSQAE";
const SOFT_NAME: &str = "rommatcher";

// ---------------------------------------------------------------------------
// User credentials (optional, stored in settings.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SsUserCredentials {
    pub username: String,
    pub password: String,
}

// ---------------------------------------------------------------------------
// Parsed game data from ScreenScraper response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct SsGameData {
    pub game_id: Option<i64>,
    pub name: Option<String>,
    pub synopsis: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>,
    pub release_date: Option<String>,
    pub rating: Option<f64>,
    pub media: Vec<SsMedia>,
}

#[derive(Debug, Clone)]
pub struct SsMedia {
    pub media_type: String, // cover, screenshot, fanart
    pub url: String,
}

// ---------------------------------------------------------------------------
// Cache helpers
// ---------------------------------------------------------------------------

pub async fn is_cached(pool: &SqlitePool, rom_id: i64) -> bool {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM screenscraper_cache WHERE rom_id = ?",
    )
    .bind(rom_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0)
        > 0
}

pub async fn save_to_cache(
    pool: &SqlitePool,
    rom_id: i64,
    game_id: Option<i64>,
    raw_response: &str,
) {
    if let Err(e) = sqlx::query(
        "INSERT INTO screenscraper_cache (rom_id, screenscraper_game_id, raw_response)
         VALUES (?, ?, ?)
         ON CONFLICT(rom_id) DO UPDATE SET
           screenscraper_game_id = excluded.screenscraper_game_id,
           raw_response = excluded.raw_response,
           fetched_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(rom_id)
    .bind(game_id)
    .bind(raw_response)
    .execute(pool)
    .await
    {
        log::warn!(
            "Failed to save ScreenScraper cache for rom {rom_id}: {e}"
        );
    }
}

// ---------------------------------------------------------------------------
// API lookup
// ---------------------------------------------------------------------------

/// Look up a game on ScreenScraper.
///
/// Tries MD5 hash first (if provided), falls back to ROM name + system ID.
/// Returns `Ok(None)` if no match found.
pub async fn lookup_game(
    client: &Client,
    user_creds: Option<&SsUserCredentials>,
    md5: Option<&str>,
    rom_name: &str,
    system_id: i64,
) -> AppResult<Option<SsGameData>> {
    // Rate limit: 1 request per second
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut params: Vec<(&str, String)> = vec![
        ("devid", DEV_ID.to_string()),
        ("devpassword", DEV_PASSWORD.to_string()),
        ("softname", SOFT_NAME.to_string()),
        ("output", "json".to_string()),
        ("systemeid", system_id.to_string()),
        ("romnom", rom_name.to_string()),
    ];

    if let Some(hash) = md5 {
        if !hash.is_empty() {
            params.push(("md5", hash.to_string()));
        }
    }

    if let Some(creds) = user_creds {
        if !creds.username.is_empty() {
            params.push(("ssid", creds.username.clone()));
            params.push(("sspassword", creds.password.clone()));
        }
    }

    let resp = client
        .get("https://api.screenscraper.fr/api2/jeuInfos.php")
        .query(&params)
        .send()
        .await
        .map_err(|e| AppError::Other(format!("ScreenScraper request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        // 404 / 430 = game not found, not an error
        if status.as_u16() == 404 || status.as_u16() == 430 {
            return Ok(None);
        }
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Other(format!(
            "ScreenScraper API returned {status}: {body}"
        )));
    }

    let body = resp.text().await.map_err(|e| {
        AppError::Other(format!("Failed to read ScreenScraper response: {e}"))
    })?;

    // ScreenScraper returns plain text errors even with 200 status
    if body.starts_with("Erreur") || body.starts_with("API closed") {
        log::warn!("ScreenScraper returned error text: {}", &body[..body.len().min(200)]);
        return Ok(None);
    }

    let parsed = parse_response(&body);
    Ok(parsed)
}

// ---------------------------------------------------------------------------
// Response parsing
// ---------------------------------------------------------------------------

/// Parse the deeply nested ScreenScraper JSON response.
fn parse_response(body: &str) -> Option<SsGameData> {
    let root: serde_json::Value = serde_json::from_str(body).ok()?;
    let jeu = root.get("response")?.get("jeu")?;

    let game_id = jeu.get("id").and_then(|v| {
        v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
    });

    let name = extract_regional_text(jeu.get("noms")?, &["us", "wor", "eu", "jp"]);

    let synopsis = jeu
        .get("synopsis")
        .and_then(|arr| extract_lang_text(arr, &["en", "us"]));

    let developer = jeu
        .get("developpeur")
        .and_then(|v| v.get("text"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let publisher = jeu
        .get("editeur")
        .and_then(|v| v.get("text"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let genre = jeu.get("genres").and_then(|genres| {
        if let Some(arr) = genres.as_array() {
            let names: Vec<String> = arr
                .iter()
                .filter_map(|g| {
                    g.get("noms").and_then(|noms| {
                        extract_regional_text(noms, &["en", "us"])
                    })
                })
                .collect();
            if names.is_empty() {
                None
            } else {
                Some(names.join(", "))
            }
        } else {
            None
        }
    });

    let release_date = jeu.get("dates").and_then(|dates| {
        if let Some(arr) = dates.as_array() {
            // Prefer US/world region
            for region in &["us", "wor", "eu"] {
                for item in arr {
                    if item.get("region").and_then(|r| r.as_str()) == Some(region) {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            return Some(text.to_string());
                        }
                    }
                }
            }
            // Fallback to first
            arr.first()
                .and_then(|d| d.get("text"))
                .and_then(|t| t.as_str())
                .map(String::from)
        } else {
            None
        }
    });

    let rating = jeu
        .get("note")
        .and_then(|v| v.get("text"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .map(|r| {
            // ScreenScraper rating is typically 0-20, scale to 0-100
            if r <= 20.0 { r * 5.0 } else { r }
        });

    // Parse media
    let mut media = Vec::new();
    if let Some(medias) = jeu.get("medias").and_then(|m| m.as_array()) {
        for m in medias {
            let ss_type = m.get("type").and_then(|t| t.as_str()).unwrap_or("");
            let url = m.get("url").and_then(|u| u.as_str()).unwrap_or("");
            if url.is_empty() {
                continue;
            }

            let art_type = match ss_type {
                "box-2D" | "box-2D-front" => Some("cover"),
                "ss" | "sstitle" => Some("screenshot"),
                "fanart" => Some("fanart"),
                _ => None,
            };

            if let Some(art_type) = art_type {
                media.push(SsMedia {
                    media_type: art_type.to_string(),
                    url: url.to_string(),
                });
            }
        }
    }

    Some(SsGameData {
        game_id,
        name,
        synopsis,
        developer,
        publisher,
        genre,
        release_date,
        rating,
        media,
    })
}

/// Extract text from a ScreenScraper regional array, preferring the given regions.
fn extract_regional_text(value: &serde_json::Value, preferred: &[&str]) -> Option<String> {
    let arr = value.as_array()?;
    for region in preferred {
        for item in arr {
            if item.get("region").and_then(|r| r.as_str()) == Some(region) {
                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                    if !text.is_empty() {
                        return Some(text.to_string());
                    }
                }
            }
        }
    }
    // Fallback: first entry
    arr.first()
        .and_then(|d| d.get("text"))
        .and_then(|t| t.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// Extract text from a ScreenScraper language-keyed array (synopsis etc).
fn extract_lang_text(value: &serde_json::Value, preferred: &[&str]) -> Option<String> {
    // Could be array of objects with "langue" key, or an object with lang keys
    if let Some(arr) = value.as_array() {
        for lang in preferred {
            for item in arr {
                if item.get("langue").and_then(|l| l.as_str()) == Some(lang) {
                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                        if !text.is_empty() {
                            return Some(text.to_string());
                        }
                    }
                }
            }
        }
        // Fallback: first
        arr.first()
            .and_then(|d| d.get("text"))
            .and_then(|t| t.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from)
    } else if let Some(obj) = value.as_object() {
        for lang in preferred {
            if let Some(text) = obj.get(*lang).and_then(|t| t.as_str()) {
                if !text.is_empty() {
                    return Some(text.to_string());
                }
            }
        }
        obj.values()
            .next()
            .and_then(|t| t.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Connection test
// ---------------------------------------------------------------------------

pub async fn test_connection(
    client: &Client,
    user_creds: &SsUserCredentials,
) -> AppResult<crate::models::SsTestResult> {
    let params: Vec<(&str, &str)> = vec![
        ("devid", DEV_ID),
        ("devpassword", DEV_PASSWORD),
        ("softname", SOFT_NAME),
        ("output", "json"),
        ("ssid", &user_creds.username),
        ("sspassword", &user_creds.password),
    ];

    let resp = client
        .get("https://api.screenscraper.fr/api2/ssuserInfos.php")
        .query(&params)
        .send()
        .await;

    match resp {
        Ok(r) => {
            if r.status().is_success() {
                let body = r.text().await.unwrap_or_default();
                if body.starts_with("Erreur") || body.contains("\"ssuser\":null") {
                    Ok(crate::models::SsTestResult {
                        success: false,
                        message: "Invalid credentials".to_string(),
                    })
                } else {
                    Ok(crate::models::SsTestResult {
                        success: true,
                        message: format!("Connected as {}", user_creds.username),
                    })
                }
            } else {
                let status = r.status();
                let body = r.text().await.unwrap_or_default();
                log::warn!("ScreenScraper test_connection failed: status={status}, body={body}");
                Ok(crate::models::SsTestResult {
                    success: false,
                    message: format!("API returned status {status}: {}", body.chars().take(200).collect::<String>()),
                })
            }
        }
        Err(e) => Ok(crate::models::SsTestResult {
            success: false,
            message: format!("Connection failed: {e}"),
        }),
    }
}
