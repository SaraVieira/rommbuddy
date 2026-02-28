use reqwest::Client;
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::models::{Achievement, AchievementData, RaTestResult};
use crate::platform_registry;

const RA_API_BASE: &str = "https://retroachievements.org/API";

/// Search RA's game list (with hashes) to find a game ID matching our ROM's MD5.
pub async fn find_game_id_by_hash(
    client: &Client,
    username: &str,
    api_key: &str,
    platform_slug: &str,
    md5: &str,
) -> Option<String> {
    let console_id = platform_registry::ra_console_id(platform_slug)?;
    log::info!("[RA] find_game_id_by_hash: platform={platform_slug} console_id={console_id} md5={md5}");
    let url = format!(
        "{RA_API_BASE}/API_GetGameList.php?z={username}&y={api_key}&i={console_id}&h=1&f=1",
    );

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            log::error!("[RA] find_game_id_by_hash: HTTP request failed: {e}");
            return None;
        }
    };
    log::info!("[RA] find_game_id_by_hash: response status={}", resp.status());
    let body_text = match resp.text().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("[RA] find_game_id_by_hash: failed to read body: {e}");
            return None;
        }
    };
    log::info!("[RA] find_game_id_by_hash: body length={}, first 500 chars: {}", body_text.len(), &body_text[..body_text.len().min(500)]);
    let games: Value = match serde_json::from_str(&body_text) {
        Ok(v) => v,
        Err(e) => {
            log::error!("[RA] find_game_id_by_hash: JSON parse failed: {e}");
            return None;
        }
    };
    let games_arr = match games.as_array() {
        Some(a) => a,
        None => {
            log::error!("[RA] find_game_id_by_hash: response is not an array, type: {}",
                if games.is_object() { "object" } else if games.is_string() { "string" } else { "other" });
            return None;
        }
    };
    log::info!("[RA] find_game_id_by_hash: got {} games from RA for console {console_id}", games_arr.len());

    let md5_lower = md5.to_lowercase();

    for game in games_arr {
        if let Some(hashes) = game["Hashes"].as_array() {
            for hash in hashes {
                if let Some(h) = hash.as_str() {
                    if h.to_lowercase() == md5_lower {
                        return game["ID"].as_u64().map(|id| id.to_string());
                    }
                }
            }
        }
    }

    None
}

pub async fn fetch_game_achievements(
    client: &Client,
    username: &str,
    api_key: &str,
    ra_game_id: &str,
) -> AppResult<AchievementData> {
    let url = format!(
        "{RA_API_BASE}/API_GetGameInfoAndUserProgress.php?z={username}&y={api_key}&u={username}&g={ra_game_id}",
    );

    let resp = client.get(&url).send().await?;
    let body: Value = resp
        .json()
        .await
        .map_err(|e| AppError::Other(format!("Failed to parse RA response: {e}")))?;

    let game_title = body["Title"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    let achievements_obj = body["Achievements"]
        .as_object()
        .cloned()
        .unwrap_or_default();

    let mut achievements: Vec<Achievement> = Vec::new();
    let mut num_earned: u32 = 0;

    for (_key, ach) in &achievements_obj {
        let earned =
            ach["DateEarned"].as_str().is_some() || ach["DateEarnedHardcore"].as_str().is_some();

        if earned {
            num_earned += 1;
        }

        let badge_id = ach["BadgeName"].as_str().unwrap_or("00000");
        let badge_url = format!("https://media.retroachievements.org/Badge/{badge_id}.png");

        let earned_date = ach["DateEarnedHardcore"]
            .as_str()
            .or_else(|| ach["DateEarned"].as_str())
            .map(|s| s.to_string());

        achievements.push(Achievement {
            id: ach["ID"].as_u64().unwrap_or(0),
            title: ach["Title"].as_str().unwrap_or("").to_string(),
            description: ach["Description"].as_str().unwrap_or("").to_string(),
            points: ach["Points"].as_u64().unwrap_or(0) as u32,
            badge_url,
            earned,
            earned_date,
        });
    }

    // Sort: earned first, then by points descending
    achievements.sort_by(|a, b| b.earned.cmp(&a.earned).then_with(|| b.points.cmp(&a.points)));

    Ok(AchievementData {
        game_title,
        num_achievements: achievements.len() as u32,
        num_earned,
        achievements,
    })
}

pub async fn test_connection(client: &Client, username: &str, api_key: &str) -> RaTestResult {
    let url = format!(
        "{RA_API_BASE}/API_GetUserSummary.php?z={username}&y={api_key}&u={username}",
    );

    match client.get(&url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<Value>().await {
                    Ok(body) => {
                        if body.get("UserPic").is_some() {
                            RaTestResult {
                                success: true,
                                message: format!("Connected as {username}"),
                            }
                        } else {
                            RaTestResult {
                                success: false,
                                message: "Invalid API key or username".to_string(),
                            }
                        }
                    }
                    Err(_) => RaTestResult {
                        success: false,
                        message: "Invalid response from RetroAchievements API".to_string(),
                    },
                }
            } else {
                RaTestResult {
                    success: false,
                    message: format!("API returned status {}", resp.status()),
                }
            }
        }
        Err(e) => RaTestResult {
            success: false,
            message: format!("Connection failed: {e}"),
        },
    }
}
