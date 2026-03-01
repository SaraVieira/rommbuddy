use sea_orm::DatabaseConnection;

/// Extract an ID value from a `serde_json::Value`, returning it as a String
/// whether it was stored as a string or a number.
fn extract_id_string(value: &serde_json::Value) -> String {
    value.as_str().map_or_else(|| value.to_string(), str::to_string)
}

/// Result from a Hasheous API lookup.
pub struct HasheousResult {
    pub hasheous_id: Option<i64>,
    pub name: String,
    pub publisher: Option<String>,
    pub year: Option<String>,
    pub description: Option<String>,
    pub genres: Vec<String>,
    pub igdb_game_id: Option<String>,
    pub igdb_platform_id: Option<String>,
    pub thegamesdb_game_id: Option<String>,
    pub retroachievements_game_id: Option<String>,
    pub retroachievements_platform_id: Option<String>,
    pub wikipedia_url: Option<String>,
    pub raw_response: String,
}

/// Look up a ROM by its MD5 hash via the Hasheous public API.
/// Returns `None` on 404 or network error.
pub async fn lookup_by_md5(client: &reqwest::Client, md5: &str) -> Option<HasheousResult> {
    let url = format!(
        "https://hasheous.org/api/v1/Lookup/ByHash/md5/{md5}",
    );

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            log::warn!("Hasheous HTTP request failed for md5 {md5}: {e}");
            return None;
        }
    };
    if !resp.status().is_success() {
        return None;
    }

    let raw_response = match resp.text().await {
        Ok(t) => t,
        Err(e) => {
            log::warn!("Failed to read Hasheous response body for md5 {md5}: {e}");
            return None;
        }
    };
    let v: serde_json::Value = match serde_json::from_str(&raw_response) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Failed to parse Hasheous JSON for md5 {md5}: {e}");
            return None;
        }
    };

    let hasheous_id = v.get("id").and_then(serde_json::Value::as_i64);
    let name = v.get("name").and_then(serde_json::Value::as_str)?.to_string();
    let publisher = v
        .pointer("/publisher/name")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let year = v
        .pointer("/signature/game/year")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);

    // Extract metadata source IDs
    let mut igdb_game_id = None;
    let mut igdb_platform_id = None;
    let mut thegamesdb_game_id = None;
    let mut retroachievements_game_id = None;
    let mut retroachievements_platform_id = None;
    let mut wikipedia_url = None;

    if let Some(metadata) = v.get("metadata").and_then(serde_json::Value::as_array) {
        for entry in metadata {
            let source = entry.get("source").and_then(serde_json::Value::as_str).unwrap_or("");
            let object_type = entry.get("objectType").and_then(serde_json::Value::as_str).unwrap_or("");
            let id_val = entry.get("id").map(extract_id_string);

            match (source, object_type) {
                ("IGDB", "Game") => igdb_game_id = id_val,
                ("TheGamesDb", "Game") => thegamesdb_game_id = id_val,
                ("RetroAchievements", "Game") => retroachievements_game_id = id_val,
                ("Wikipedia", _) => {
                    wikipedia_url = id_val;
                }
                _ => {}
            }
        }
    }

    // Platform metadata for IGDB and `RetroAchievements` platform IDs
    if let Some(platform_meta) = v.pointer("/platform/metadata").and_then(serde_json::Value::as_array) {
        for entry in platform_meta {
            let source = entry.get("source").and_then(serde_json::Value::as_str).unwrap_or("");
            let id_val = entry.get("id").map(extract_id_string);
            match source {
                "IGDB" => igdb_platform_id = id_val,
                "RetroAchievements" => retroachievements_platform_id = id_val,
                _ => {}
            }
        }
    }

    // Extract description from attributes (`AIDescription`)
    let mut description = None;
    let mut genres = Vec::new();

    if let Some(attributes) = v.get("attributes").and_then(serde_json::Value::as_array) {
        for attr in attributes {
            let attr_name = attr.get("attributeName").and_then(serde_json::Value::as_str).unwrap_or("");
            match attr_name {
                "AIDescription" => {
                    description = attr.get("value").and_then(serde_json::Value::as_str).map(str::to_string);
                }
                "Tags" => {
                    // value.GameGenre.Tags is an array of {Text: "..."}
                    if let Some(tags) = attr.pointer("/value/GameGenre/Tags").and_then(serde_json::Value::as_array) {
                        for tag in tags {
                            if let Some(text) = tag.get("Text").and_then(serde_json::Value::as_str) {
                                genres.push(text.to_string());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Some(HasheousResult {
        hasheous_id,
        name,
        publisher,
        year,
        description,
        genres,
        igdb_game_id,
        igdb_platform_id,
        thegamesdb_game_id,
        retroachievements_game_id,
        retroachievements_platform_id,
        wikipedia_url,
        raw_response,
    })
}

/// Save a Hasheous result to the `hasheous_cache` table.
pub async fn save_to_cache(db: &DatabaseConnection, rom_id: i64, result: &HasheousResult) {
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

    let genres_json = serde_json::to_string(&result.genres).unwrap_or_else(|_| "[]".to_string());

    if let Err(e) = db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "INSERT INTO hasheous_cache (
            rom_id, hasheous_id, name, publisher, year, description, genres,
            igdb_game_id, igdb_platform_id, thegamesdb_game_id,
            retroachievements_game_id, retroachievements_platform_id,
            wikipedia_url, raw_response
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(rom_id) DO UPDATE SET
            hasheous_id = excluded.hasheous_id,
            name = excluded.name,
            publisher = excluded.publisher,
            year = excluded.year,
            description = excluded.description,
            genres = excluded.genres,
            igdb_game_id = excluded.igdb_game_id,
            igdb_platform_id = excluded.igdb_platform_id,
            thegamesdb_game_id = excluded.thegamesdb_game_id,
            retroachievements_game_id = excluded.retroachievements_game_id,
            retroachievements_platform_id = excluded.retroachievements_platform_id,
            wikipedia_url = excluded.wikipedia_url,
            raw_response = excluded.raw_response,
            fetched_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
            [
                rom_id.into(),
                result.hasheous_id.into(),
                result.name.clone().into(),
                result.publisher.clone().into(),
                result.year.clone().into(),
                result.description.clone().into(),
                genres_json.into(),
                result.igdb_game_id.clone().into(),
                result.igdb_platform_id.clone().into(),
                result.thegamesdb_game_id.clone().into(),
                result.retroachievements_game_id.clone().into(),
                result.retroachievements_platform_id.clone().into(),
                result.wikipedia_url.clone().into(),
                result.raw_response.clone().into(),
            ],
        ))
        .await
    {
        log::warn!("Failed to save Hasheous cache for rom {rom_id}: {e}");
    }
}

/// Check if we already have a cached Hasheous result for a ROM.
pub async fn get_cached(db: &DatabaseConnection, rom_id: i64) -> Option<HasheousResult> {
    use crate::entity::hasheous_cache::{self, Column};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let model = match hasheous_cache::Entity::find()
        .filter(Column::RomId.eq(rom_id))
        .one(db)
        .await
    {
        Ok(Some(m)) => m,
        Ok(None) => return None,
        Err(e) => {
            log::warn!("Failed to query Hasheous cache for rom {rom_id}: {e}");
            return None;
        }
    };

    Some(HasheousResult {
        hasheous_id: model.hasheous_id,
        name: model.name?,
        publisher: model.publisher,
        year: model.year,
        description: model.description,
        genres: model.genres.into_inner(),
        igdb_game_id: model.igdb_game_id,
        igdb_platform_id: model.igdb_platform_id,
        thegamesdb_game_id: model.thegamesdb_game_id,
        retroachievements_game_id: model.retroachievements_game_id,
        retroachievements_platform_id: model.retroachievements_platform_id,
        wikipedia_url: model.wikipedia_url,
        raw_response: model.raw_response.unwrap_or_default(),
    })
}
