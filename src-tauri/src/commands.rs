use std::collections::HashMap;

use futures_util::StreamExt;
use sea_orm::DatabaseConnection;
use tauri::ipc::Channel;
use tauri::State;
use tauri_plugin_store::StoreExt;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

use crate::error::{AppError, AppResult};
use crate::platform_registry;
use crate::models::{
    AchievementData, CacheInfo, CachedFile, ConnectionTestResult, CoreInfo, CoreMapping,
    DownloadProgress, EmulatorDef, IgdbTestResult, LibraryPage, Platform, PlatformWithCount,
    RaTestResult, RomWithMeta, SaveFileInfo, SavePathOverride, ScanProgress, SsTestResult,
    SourceConfig,
};
use crate::saves;
use crate::sources::local_sync;
use crate::sources::romm::RommClient;

pub(crate) fn rom_cache_dir() -> std::path::PathBuf {
    directories::ProjectDirs::from("com", "romm-buddy", "romm-buddy")
        .map_or_else(
            || std::path::PathBuf::from("rom_cache"),
            |p| p.cache_dir().join("rom_cache"),
        )
}

struct EmulatorEntry {
    id: &'static str,
    name: &'static str,
    default_macos_app: &'static str,
    platforms: &'static [&'static str],
}

const EMULATOR_REGISTRY: &[EmulatorEntry] = &[
    EmulatorEntry {
        id: "dolphin",
        name: "Dolphin",
        default_macos_app: "/Applications/Dolphin.app",
        platforms: &["gc", "wii"],
    },
    EmulatorEntry {
        id: "duckstation",
        name: "DuckStation",
        default_macos_app: "/Applications/DuckStation.app",
        platforms: &["psx"],
    },
    EmulatorEntry {
        id: "pcsx2",
        name: "PCSX2",
        default_macos_app: "/Applications/PCSX2.app",
        platforms: &["ps2"],
    },
    EmulatorEntry {
        id: "mgba",
        name: "mGBA",
        default_macos_app: "/Applications/mGBA.app",
        platforms: &["gba", "gb", "gbc"],
    },
    EmulatorEntry {
        id: "cemu",
        name: "Cemu",
        default_macos_app: "/Applications/Cemu.app",
        platforms: &["wiiu"],
    },
    EmulatorEntry {
        id: "xemu",
        name: "xemu",
        default_macos_app: "/Applications/xemu.app",
        platforms: &["xbox"],
    },
    EmulatorEntry {
        id: "rpcs3",
        name: "RPCS3",
        default_macos_app: "/Applications/RPCS3.app",
        platforms: &["ps3"],
    },
    EmulatorEntry {
        id: "melonds",
        name: "melonDS",
        default_macos_app: "/Applications/melonDS.app",
        platforms: &["nds"],
    },
];

fn build_emulator_args(emulator_type: &str, rom_path: &str) -> Vec<String> {
    match emulator_type {
        "dolphin" => vec![format!("--exec={rom_path}")],
        "duckstation" | "pcsx2" => vec![rom_path.into()],
        "cemu" => vec!["-g".into(), rom_path.into()],
        "xemu" => vec!["-dvd_path".into(), rom_path.into()],
        "rpcs3" => vec!["--no-gui".into(), rom_path.into()],
        _ => vec![rom_path.into()],
    }
}

#[tauri::command]
pub async fn get_platforms(db: State<'_, DatabaseConnection>) -> AppResult<Vec<Platform>> {
    use crate::entity::platforms;
    use sea_orm::{EntityTrait, QueryOrder};

    let models = platforms::Entity::find()
        .order_by_asc(platforms::Column::Name)
        .all(db.inner())
        .await?;

    Ok(models
        .into_iter()
        .map(|m| Platform {
            id: m.id,
            slug: m.slug,
            name: m.name,
            igdb_id: m.igdb_id,
            file_extensions: serde_json::from_str(&m.file_extensions).unwrap_or_default(),
        })
        .collect())
}

#[tauri::command]
pub async fn get_sources(db: State<'_, DatabaseConnection>) -> AppResult<Vec<SourceConfig>> {
    use crate::entity::sources;
    use sea_orm::{EntityTrait, QueryOrder};

    let models = sources::Entity::find()
        .order_by_asc(sources::Column::Name)
        .all(db.inner())
        .await?;

    Ok(models
        .into_iter()
        .map(|m| SourceConfig {
            id: m.id,
            name: m.name,
            source_type: match m.source_type.as_str() {
                "romm" => crate::models::SourceType::Romm,
                _ => crate::models::SourceType::Local,
            },
            url: m.url,
            enabled: m.enabled != 0,
            last_synced_at: m.last_synced_at,
            created_at: m.created_at.parse().unwrap_or_default(),
            updated_at: m.updated_at.parse().unwrap_or_default(),
        })
        .collect())
}

#[tauri::command]
pub async fn test_romm_connection(
    url: String,
    username: String,
    password: String,
) -> AppResult<ConnectionTestResult> {
    let client = RommClient::new(url, username, password);
    client.test_connection().await
}

#[tauri::command]
pub async fn test_local_path(path: String) -> AppResult<ConnectionTestResult> {
    let root = std::path::Path::new(&path);
    let (_layout, platform_count, rom_count) = local_sync::test_local_path(root)?;
    #[allow(clippy::cast_possible_truncation)]
    Ok(ConnectionTestResult {
        platform_count,
        rom_count: rom_count as u32,
    })
}

#[tauri::command]
pub async fn add_source(
    db: State<'_, DatabaseConnection>,
    name: String,
    source_type: String,
    url: Option<String>,
    credentials_json: String,
) -> AppResult<i64> {
    use crate::entity::sources;
    use sea_orm::{ActiveModelTrait, ActiveValue::Set};

    let model = sources::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        name: Set(name),
        source_type: Set(source_type),
        url: Set(url),
        credentials: Set(credentials_json),
        settings: Set("{}".to_string()),
        enabled: Set(1),
        last_synced_at: Set(None),
        created_at: Set(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
        updated_at: Set(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
    }
    .insert(db.inner())
    .await?;

    Ok(model.id)
}

#[tauri::command]
pub async fn update_source(
    db: State<'_, DatabaseConnection>,
    source_id: i64,
    name: String,
    url: Option<String>,
    credentials_json: String,
) -> AppResult<()> {
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

    db.inner()
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "UPDATE sources SET name = ?, url = ?, credentials = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?",
            [name.into(), url.into(), credentials_json.into(), source_id.into()],
        ))
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_source_credentials(
    db: State<'_, DatabaseConnection>,
    source_id: i64,
) -> AppResult<String> {
    use crate::entity::sources;
    use sea_orm::EntityTrait;

    let model = sources::Entity::find_by_id(source_id)
        .one(db.inner())
        .await?
        .ok_or_else(|| AppError::SourceNotFound(source_id.to_string()))?;
    Ok(model.credentials)
}

#[tauri::command]
pub async fn remove_source(
    db: State<'_, DatabaseConnection>,
    source_id: i64,
) -> AppResult<()> {
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, TransactionTrait};

    let txn = db.inner().begin().await?;

    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "DELETE FROM source_roms WHERE source_id = ?",
        [source_id.into()],
    ))
    .await?;
    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "DELETE FROM library WHERE source_id = ?",
        [source_id.into()],
    ))
    .await?;
    // Clean up orphaned roms (no remaining source_roms linking to them)
    // ON DELETE CASCADE on metadata/artwork/roms_fts handles the rest
    txn.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "DELETE FROM roms WHERE id NOT IN (SELECT DISTINCT rom_id FROM source_roms)",
    ))
    .await?;
    txn.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "DELETE FROM sources WHERE id = ?",
        [source_id.into()],
    ))
    .await?;

    txn.commit().await?;
    Ok(())
}

#[tauri::command]
pub async fn sync_source(
    db: State<'_, DatabaseConnection>,
    cancel_tokens: State<'_, CancelTokenMap>,
    source_id: i64,
    channel: Channel<ScanProgress>,
) -> AppResult<()> {
    // Get source info
    use crate::entity::sources;
    use sea_orm::EntityTrait;

    let source = sources::Entity::find_by_id(source_id)
        .one(db.inner())
        .await?
        .ok_or_else(|| AppError::SourceNotFound(source_id.to_string()))?;

    let (url_opt, credentials, source_type) = (source.url, source.credentials, source.source_type);

    let cancel = CancellationToken::new();
    cancel_tokens
        .0
        .lock()
        .await
        .insert(CancelKey::Source(source_id), cancel.clone());

    let db_ref = db.inner();

    let result = match source_type.as_str() {
        "local" => {
            let creds: HashMap<String, String> =
                serde_json::from_str(&credentials).map_err(|e| AppError::Other(e.to_string()))?;
            let path = creds
                .get("path")
                .ok_or_else(|| AppError::Other("Missing path in credentials".to_string()))?
                .clone();
            let root = std::path::PathBuf::from(path);
            local_sync::sync_local_to_db(source_id, &root, db_ref, move |progress| {
                let _ = channel.send(progress);
            }, cancel)
            .await
        }
        "romm" => {
            let url = url_opt.ok_or_else(|| {
                AppError::Other("Source has no URL configured".to_string())
            })?;
            let creds: HashMap<String, String> =
                serde_json::from_str(&credentials).map_err(|e| AppError::Other(e.to_string()))?;
            let username = creds
                .get("username")
                .ok_or_else(|| AppError::Other("Missing username in credentials".to_string()))?
                .clone();
            let password = creds
                .get("password")
                .ok_or_else(|| AppError::Other("Missing password in credentials".to_string()))?
                .clone();
            let client = RommClient::new(url, username, password);
            client.sync_to_db(source_id, db_ref, move |progress| {
                let _ = channel.send(progress);
            }, cancel)
            .await
        }
        other => {
            Err(AppError::Other(format!("Unknown source type: {other}")))
        }
    };

    cancel_tokens.0.lock().await.remove(&CancelKey::Source(source_id));
    result
}

#[tauri::command]
pub async fn cancel_sync(
    cancel_tokens: State<'_, CancelTokenMap>,
    source_id: i64,
) -> AppResult<()> {
    if let Some(token) = cancel_tokens.0.lock().await.get(&CancelKey::Source(source_id)) {
        token.cancel();
    }
    Ok(())
}

#[derive(Debug, sea_orm::FromQueryResult)]
struct RomWithMetaRow {
    id: i64,
    platform_id: i64,
    platform_slug: String,
    platform_name: String,
    name: String,
    file_name: String,
    file_size: Option<i64>,
    regions: String,
    description: Option<String>,
    rating: Option<f64>,
    release_date: Option<String>,
    developer: Option<String>,
    publisher: Option<String>,
    genres: String,
    themes: String,
    languages: String,
    cover_url: Option<String>,
    retroachievements_game_id: Option<String>,
    wikipedia_url: Option<String>,
    igdb_id: Option<i64>,
    thegamesdb_game_id: Option<String>,
    source_id: i64,
    source_rom_id: Option<String>,
    source_type: Option<String>,
    favorite: i64,
    verification_status: Option<String>,
    dat_game_name: Option<String>,
}

impl RomWithMetaRow {
    fn into_rom_with_meta(self) -> RomWithMeta {
        RomWithMeta {
            id: self.id,
            platform_id: self.platform_id,
            platform_slug: self.platform_slug,
            platform_name: self.platform_name,
            name: self.name,
            file_name: self.file_name,
            file_size: self.file_size,
            regions: serde_json::from_str(&self.regions).unwrap_or_default(),
            description: self.description,
            rating: self.rating,
            release_date: self.release_date,
            developer: self.developer,
            publisher: self.publisher,
            genres: serde_json::from_str(&self.genres).unwrap_or_default(),
            themes: serde_json::from_str(&self.themes).unwrap_or_default(),
            languages: serde_json::from_str(&self.languages).unwrap_or_default(),
            cover_url: self.cover_url,
            screenshot_urls: vec![],
            source_id: self.source_id,
            source_rom_id: self.source_rom_id,
            source_type: self.source_type,
            retroachievements_game_id: self.retroachievements_game_id,
            wikipedia_url: self.wikipedia_url,
            igdb_id: self.igdb_id,
            thegamesdb_game_id: self.thegamesdb_game_id,
            favorite: self.favorite != 0,
            verification_status: self.verification_status,
            dat_game_name: self.dat_game_name,
        }
    }
}

const ROM_WITH_META_SELECT: &str =
    "SELECT r.id, r.platform_id, p.slug as platform_slug, p.name as platform_name,
            r.name, r.file_name, r.file_size, r.regions,
            m.description, m.rating, m.release_date, m.developer, m.publisher,
            COALESCE(m.genres, '[]') as genres,
            COALESCE(m.themes, '[]') as themes,
            COALESCE(r.languages, '[]') as languages,
            (SELECT url FROM artwork WHERE rom_id = r.id AND art_type = 'cover' LIMIT 1) as cover_url,
            hc.retroachievements_game_id,
            hc.wikipedia_url,
            m.igdb_id,
            hc.thegamesdb_game_id,
            sr.source_id, sr.source_rom_id, s.source_type,
            COALESCE((SELECT MAX(favorite) FROM library l WHERE l.rom_id = r.id), 0) as favorite,
            r.verification_status, r.dat_game_name
     FROM roms r
     JOIN platforms p ON p.id = r.platform_id";

/// Default library sort: last-played first (most recent on top), then a
/// deterministic pseudo-random shuffle for everything else (stable across pages).
const LIBRARY_ORDER: &str =
    "(SELECT MAX(l.last_played_at) FROM library l WHERE l.rom_id = r.id) IS NULL,
     (SELECT MAX(l.last_played_at) FROM library l WHERE l.rom_id = r.id) DESC,
     (r.id * 2654435761) % 4294967296";

/// Helper: execute a raw count query with dynamic values via SeaORM.
async fn count_query(db: &DatabaseConnection, sql: &str, values: Vec<sea_orm::Value>) -> AppResult<i64> {
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
    let result = db.query_one(Statement::from_sql_and_values(DatabaseBackend::Sqlite, sql, values))
        .await?
        .ok_or_else(|| crate::error::AppError::Other("count query returned no rows".to_string()))?;
    Ok(result.try_get::<i64>("", "COUNT(*)")
        .or_else(|_| result.try_get_by_index::<i64>(0))?)
}

/// Helper: execute a RomWithMetaRow query via SeaORM.
async fn query_rom_rows(db: &DatabaseConnection, sql: &str, values: Vec<sea_orm::Value>) -> AppResult<Vec<RomWithMetaRow>> {
    use sea_orm::{DatabaseBackend, FromQueryResult, Statement};
    let stmt = Statement::from_sql_and_values(DatabaseBackend::Sqlite, sql, values);
    Ok(RomWithMetaRow::find_by_statement(stmt).all(db).await?)
}

#[tauri::command]
pub async fn get_library_roms(
    db: State<'_, DatabaseConnection>,
    platform_id: Option<i64>,
    search: Option<String>,
    favorites_only: Option<bool>,
    offset: i64,
    limit: i64,
) -> AppResult<LibraryPage> {
    let favorites_only = favorites_only.unwrap_or(false);

    // Build query based on filters
    let (rows, total) = if let Some(ref query) = search {
        if query.trim().is_empty() {
            return get_library_roms_filtered(db, platform_id, favorites_only, offset, limit)
                .await;
        }
        // FTS search
        let search_query = format!("{}*", query.replace('"', ""));

        let fav_clause = if favorites_only {
            " AND EXISTS (SELECT 1 FROM library l WHERE l.rom_id = r.id AND l.favorite = 1)"
        } else {
            ""
        };

        let count = if let Some(pid) = platform_id {
            let q = format!(
                "SELECT COUNT(*) FROM roms r
                 JOIN roms_fts ON roms_fts.rowid = r.id
                 WHERE roms_fts MATCH ? AND r.platform_id = ?{fav_clause}"
            );
            count_query(db.inner(), &q, vec![search_query.clone().into(), pid.into()]).await?
        } else {
            let q = format!(
                "SELECT COUNT(*) FROM roms r
                 JOIN roms_fts ON roms_fts.rowid = r.id
                 WHERE roms_fts MATCH ?{fav_clause}"
            );
            count_query(db.inner(), &q, vec![search_query.clone().into()]).await?
        };

        let rows = if let Some(pid) = platform_id {
            let q = format!(
                "{ROM_WITH_META_SELECT} JOIN roms_fts ON roms_fts.rowid = r.id
                 LEFT JOIN metadata m ON m.rom_id = r.id

                 LEFT JOIN hasheous_cache hc ON hc.rom_id = r.id
                 LEFT JOIN source_roms sr ON sr.rom_id = r.id
                 LEFT JOIN sources s ON s.id = sr.source_id
                 WHERE roms_fts MATCH ? AND r.platform_id = ?{fav_clause}
                 GROUP BY r.id
                 ORDER BY {LIBRARY_ORDER}
                 LIMIT ? OFFSET ?",
            );
            query_rom_rows(db.inner(), &q, vec![search_query.clone().into(), pid.into(), limit.into(), offset.into()]).await?
        } else {
            let q = format!(
                "{ROM_WITH_META_SELECT} JOIN roms_fts ON roms_fts.rowid = r.id
                 LEFT JOIN metadata m ON m.rom_id = r.id

                 LEFT JOIN hasheous_cache hc ON hc.rom_id = r.id
                 LEFT JOIN source_roms sr ON sr.rom_id = r.id
                 LEFT JOIN sources s ON s.id = sr.source_id
                 WHERE roms_fts MATCH ?{fav_clause}
                 GROUP BY r.id
                 ORDER BY {LIBRARY_ORDER}
                 LIMIT ? OFFSET ?",
            );
            query_rom_rows(db.inner(), &q, vec![search_query.clone().into(), limit.into(), offset.into()]).await?
        };

        (rows, count)
    } else {
        return get_library_roms_filtered(db, platform_id, favorites_only, offset, limit).await;
    };

    Ok(LibraryPage {
        roms: rows
            .into_iter()
            .map(RomWithMetaRow::into_rom_with_meta)
            .collect(),
        total,
    })
}

async fn get_library_roms_filtered(
    db: State<'_, DatabaseConnection>,
    platform_id: Option<i64>,
    favorites_only: bool,
    offset: i64,
    limit: i64,
) -> AppResult<LibraryPage> {
    let fav_clause = if favorites_only {
        " EXISTS (SELECT 1 FROM library l WHERE l.rom_id = r.id AND l.favorite = 1)"
    } else {
        ""
    };

    let (rows, total) = if let Some(pid) = platform_id {
        let where_clause = if favorites_only {
            format!("WHERE r.platform_id = ? AND{fav_clause}")
        } else {
            "WHERE r.platform_id = ?".to_string()
        };

        let count_q = format!("SELECT COUNT(*) FROM roms r {where_clause}");
        let count = count_query(db.inner(), &count_q, vec![pid.into()]).await?;

        let q = format!(
            "{ROM_WITH_META_SELECT} LEFT JOIN metadata m ON m.rom_id = r.id
             LEFT JOIN artwork a ON a.rom_id = r.id AND a.art_type = 'cover'
             LEFT JOIN hasheous_cache hc ON hc.rom_id = r.id
             LEFT JOIN source_roms sr ON sr.rom_id = r.id
             LEFT JOIN sources s ON s.id = sr.source_id
             {where_clause}
             GROUP BY r.id
             ORDER BY {LIBRARY_ORDER}
             LIMIT ? OFFSET ?",
        );
        let rows = query_rom_rows(db.inner(), &q, vec![pid.into(), limit.into(), offset.into()]).await?;

        (rows, count)
    } else if favorites_only {
        let where_clause = format!("WHERE{fav_clause}");

        let count_q = format!("SELECT COUNT(*) FROM roms r {where_clause}");
        let count = count_query(db.inner(), &count_q, vec![]).await?;

        let q = format!(
            "{ROM_WITH_META_SELECT} LEFT JOIN metadata m ON m.rom_id = r.id
             LEFT JOIN artwork a ON a.rom_id = r.id AND a.art_type = 'cover'
             LEFT JOIN hasheous_cache hc ON hc.rom_id = r.id
             LEFT JOIN source_roms sr ON sr.rom_id = r.id
             LEFT JOIN sources s ON s.id = sr.source_id
             {where_clause}
             GROUP BY r.id
             ORDER BY {LIBRARY_ORDER}
             LIMIT ? OFFSET ?",
        );
        let rows = query_rom_rows(db.inner(), &q, vec![limit.into(), offset.into()]).await?;

        (rows, count)
    } else {
        let count = count_query(db.inner(), "SELECT COUNT(*) FROM roms", vec![]).await?;

        let q = format!(
            "{ROM_WITH_META_SELECT} LEFT JOIN metadata m ON m.rom_id = r.id
             LEFT JOIN artwork a ON a.rom_id = r.id AND a.art_type = 'cover'
             LEFT JOIN hasheous_cache hc ON hc.rom_id = r.id
             LEFT JOIN source_roms sr ON sr.rom_id = r.id
             LEFT JOIN sources s ON s.id = sr.source_id
             GROUP BY r.id
             ORDER BY {LIBRARY_ORDER}
             LIMIT ? OFFSET ?",
        );
        let rows = query_rom_rows(db.inner(), &q, vec![limit.into(), offset.into()]).await?;

        (rows, count)
    };

    Ok(LibraryPage {
        roms: rows
            .into_iter()
            .map(RomWithMetaRow::into_rom_with_meta)
            .collect(),
        total,
    })
}

#[tauri::command]
pub async fn toggle_favorite(
    db: State<'_, DatabaseConnection>,
    rom_id: i64,
    favorite: bool,
) -> AppResult<bool> {
    use sea_orm::{ConnectionTrait, DatabaseBackend, FromQueryResult, Statement};

    let fav_val: i64 = if favorite { 1 } else { 0 };

    // Upsert: if no library row exists, create one (look up source_id from source_roms)
    #[derive(Debug, FromQueryResult)]
    struct SourceIdRow {
        source_id: i64,
    }
    let source_id = SourceIdRow::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "SELECT source_id FROM source_roms WHERE rom_id = ? LIMIT 1",
        [rom_id.into()],
    ))
    .one(db.inner())
    .await?
    .map(|r| r.source_id)
    .unwrap_or(0);

    db.inner()
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "INSERT INTO library (rom_id, source_id, favorite) VALUES (?, ?, ?) ON CONFLICT(rom_id, source_id) DO UPDATE SET favorite = excluded.favorite",
            [rom_id.into(), source_id.into(), fav_val.into()],
        ))
        .await?;

    Ok(favorite)
}

#[tauri::command]
pub async fn get_favorites_count(db: State<'_, DatabaseConnection>) -> AppResult<i64> {
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

    let result = db
        .inner()
        .query_one(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT COUNT(DISTINCT rom_id) as cnt FROM library WHERE favorite = 1",
        ))
        .await?
        .ok_or_else(|| AppError::Other("Count query failed".to_string()))?;
    let count: i64 = result.try_get("", "cnt").unwrap_or(0);
    Ok(count)
}

#[tauri::command]
pub async fn get_platforms_with_counts(
    db: State<'_, DatabaseConnection>,
) -> AppResult<Vec<PlatformWithCount>> {
    use sea_orm::{DatabaseBackend, FromQueryResult, Statement};

    #[derive(Debug, FromQueryResult)]
    struct PlatformCountRow {
        id: i64,
        slug: String,
        name: String,
        rom_count: i64,
    }

    let rows = PlatformCountRow::find_by_statement(Statement::from_string(
        DatabaseBackend::Sqlite,
        "SELECT p.id, p.slug, p.name, COUNT(r.id) as rom_count FROM platforms p INNER JOIN roms r ON r.platform_id = p.id GROUP BY p.id ORDER BY p.name",
    ))
    .all(db.inner())
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| PlatformWithCount {
            id: r.id,
            slug: r.slug,
            name: r.name,
            rom_count: r.rom_count,
        })
        .collect())
}

#[tauri::command]
pub fn get_all_registry_platforms() -> Vec<(String, String)> {
    platform_registry::PLATFORMS
        .iter()
        .map(|p| (p.slug.to_string(), p.display_name.to_string()))
        .collect()
}

#[tauri::command]
pub async fn proxy_image(
    db: State<'_, DatabaseConnection>,
    url: String,
) -> AppResult<String> {
    // Get any ROMM source credentials to authenticate if needed
    use crate::entity::sources;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let romm_source = sources::Entity::find()
        .filter(sources::Column::SourceType.eq("romm"))
        .one(db.inner())
        .await?;
    let row = romm_source.map(|s| (s.url.unwrap_or_default(), s.credentials));

    if let Some((base_url, credentials)) = row {
        let creds: HashMap<String, String> =
            serde_json::from_str(&credentials).unwrap_or_default();
        let username = creds.get("username").cloned().unwrap_or_default();
        let password = creds.get("password").cloned().unwrap_or_default();
        let client = RommClient::new(base_url, username, password);
        client.proxy_image(&url).await
    } else {
        // No source, try direct fetch and return as base64 data URL
        use base64::Engine;
        let resp = reqwest::get(&url).await?;
        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/jpeg")
            .to_string();
        const MAX_IMAGE_SIZE: u64 = 10 * 1024 * 1024; // 10 MB
        if let Some(len) = resp.content_length() {
            if len > MAX_IMAGE_SIZE {
                return Err(crate::error::AppError::Other(format!(
                    "Image too large: {len} bytes (max {MAX_IMAGE_SIZE})"
                )));
            }
        }
        let bytes = resp.bytes().await?;
        if bytes.len() as u64 > MAX_IMAGE_SIZE {
            return Err(crate::error::AppError::Other(format!(
                "Image too large: {} bytes (max {MAX_IMAGE_SIZE})",
                bytes.len()
            )));
        }
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        Ok(format!("data:{content_type};base64,{b64}"))
    }
}

#[tauri::command]
pub async fn get_retroarch_path(app: tauri::AppHandle) -> AppResult<Option<String>> {
    // Try store first
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    if let Some(val) = store.get("retroarch_path") {
        if let Some(path) = val.as_str() {
            return Ok(Some(path.to_string()));
        }
    }

    // Auto-detect common paths
    let candidates = if cfg!(target_os = "macos") {
        vec!["/Applications/RetroArch.app/Contents/MacOS/RetroArch"]
    } else if cfg!(target_os = "windows") {
        vec![
            "C:\\RetroArch\\retroarch.exe",
            "C:\\Program Files\\RetroArch\\retroarch.exe",
        ]
    } else {
        vec!["/usr/bin/retroarch", "/usr/local/bin/retroarch"]
    };

    for path in candidates {
        if std::path::Path::new(path).exists() {
            return Ok(Some(path.to_string()));
        }
    }

    Ok(None)
}

#[tauri::command]
pub async fn set_retroarch_path(app: tauri::AppHandle, path: String) -> AppResult<()> {
    if !std::path::Path::new(&path).exists() {
        return Err(AppError::Other(format!("Path does not exist: {path}")));
    }
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    store.set("retroarch_path", serde_json::json!(path));
    store.save().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn get_emulators() -> AppResult<Vec<EmulatorDef>> {
    Ok(EMULATOR_REGISTRY
        .iter()
        .map(|e| EmulatorDef {
            id: e.id.to_string(),
            name: e.name.to_string(),
            platforms: e.platforms.iter().map(|s| (*s).to_string()).collect(),
        })
        .collect())
}

#[tauri::command]
pub async fn get_emulator_paths(app: tauri::AppHandle) -> AppResult<HashMap<String, String>> {
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    if let Some(val) = store.get("emulator_paths") {
        if let Some(obj) = val.as_object() {
            let mut map = HashMap::new();
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    map.insert(k.clone(), s.to_string());
                }
            }
            return Ok(map);
        }
    }
    Ok(HashMap::new())
}

#[tauri::command]
pub async fn set_emulator_path(
    app: tauri::AppHandle,
    emulator_id: String,
    path: String,
) -> AppResult<()> {
    if !std::path::Path::new(&path).exists() {
        return Err(AppError::Other(format!("Path does not exist: {path}")));
    }
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    let mut paths: serde_json::Map<String, serde_json::Value> =
        store.get("emulator_paths")
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
    paths.insert(emulator_id, serde_json::json!(path));
    store.set("emulator_paths", serde_json::json!(paths));
    store.save().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn detect_emulators() -> AppResult<Vec<(String, String)>> {
    let mut found = Vec::new();
    for entry in EMULATOR_REGISTRY {
        if std::path::Path::new(entry.default_macos_app).exists() {
            found.push((
                entry.id.to_string(),
                entry.default_macos_app.to_string(),
            ));
        }
    }
    Ok(found)
}

/// Parse `display_name` from a `RetroArch` `.info` file (simple key = "value" format).
fn parse_display_name(info_path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(info_path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("display_name") {
            let rest = rest.trim();
            if let Some(rest) = rest.strip_prefix('=') {
                let rest = rest.trim();
                let rest = rest.trim_matches('"');
                if !rest.is_empty() {
                    return Some(rest.to_string());
                }
            }
        }
    }
    None
}

/// Locate the `RetroArch` info directory (sibling to cores dir).
fn find_info_dir() -> Option<std::path::PathBuf> {
    if cfg!(target_os = "macos") {
        if let Some(home) = std::env::var_os("HOME") {
            let info_dir = std::path::PathBuf::from(home)
                .join("Library/Application Support/RetroArch/info");
            if info_dir.exists() {
                return Some(info_dir);
            }
        }
    }
    None
}

/// Locate the `RetroArch` cores directory.
fn find_cores_dir(retroarch_path: &str) -> Option<std::path::PathBuf> {
    let ra_path = std::path::Path::new(retroarch_path);
    let mut candidate_dirs: Vec<std::path::PathBuf> = Vec::new();

    if cfg!(target_os = "macos") {
        if let Some(home) = std::env::var_os("HOME") {
            let user_cores = std::path::PathBuf::from(home)
                .join("Library/Application Support/RetroArch/cores");
            candidate_dirs.push(user_cores);
        }
        if let Some(bundle_cores) = ra_path
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("Resources").join("cores"))
        {
            candidate_dirs.push(bundle_cores);
        }
    } else if let Some(p) = ra_path.parent() {
        candidate_dirs.push(p.join("cores"));
    }

    candidate_dirs.into_iter().find(|d| d.exists())
}

#[tauri::command]
pub async fn detect_cores(retroarch_path: String) -> AppResult<Vec<CoreInfo>> {
    let Some(cores_dir) = find_cores_dir(&retroarch_path) else {
        return Ok(vec![]);
    };

    let info_dir = find_info_dir();

    let ext = if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(target_os = "windows") {
        "dll"
    } else {
        "so"
    };

    let mut cores = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&cores_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some(ext) {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let display_name = info_dir.as_ref().and_then(|dir| {
                    parse_display_name(&dir.join(format!("{name}.info")))
                });
                cores.push(CoreInfo {
                    core_name: name,
                    core_path: path.to_string_lossy().to_string(),
                    display_name,
                });
            }
        }
    }
    cores.sort_by(|a, b| a.core_name.cmp(&b.core_name));
    Ok(cores)
}

#[tauri::command]
pub async fn get_core_mappings(db: State<'_, DatabaseConnection>) -> AppResult<Vec<CoreMapping>> {
    use crate::entity::core_mappings;
    use sea_orm::EntityTrait;

    let models = core_mappings::Entity::find().all(db.inner()).await?;

    Ok(models
        .into_iter()
        .map(|m| CoreMapping {
            id: m.id,
            platform_id: m.platform_id,
            core_name: m.core_name,
            core_path: m.core_path,
            is_default: m.is_default != 0,
            emulator_type: m.emulator_type,
        })
        .collect())
}

#[tauri::command]
pub async fn has_core_mapping(
    db: State<'_, DatabaseConnection>,
    platform_id: i64,
) -> AppResult<bool> {
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
    let row = db.inner().query_one(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "SELECT 1 FROM core_mappings WHERE platform_id = ? LIMIT 1",
        [platform_id.into()],
    )).await?;
    Ok(row.is_some())
}

#[tauri::command]
pub async fn set_core_mapping(
    db: State<'_, DatabaseConnection>,
    platform_id: i64,
    core_name: String,
    core_path: String,
    emulator_type: Option<String>,
) -> AppResult<()> {
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

    let emu_type = emulator_type.unwrap_or_else(|| "retroarch".to_string());
    db.inner()
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "INSERT INTO core_mappings (platform_id, core_name, core_path, is_default, emulator_type) VALUES (?, ?, ?, 1, ?) ON CONFLICT(platform_id) DO UPDATE SET core_name = excluded.core_name, core_path = excluded.core_path, emulator_type = excluded.emulator_type",
            [platform_id.into(), core_name.into(), core_path.into(), emu_type.into()],
        ))
        .await?;
    Ok(())
}

#[tauri::command]
#[allow(clippy::similar_names)]
pub async fn download_and_launch(
    app: tauri::AppHandle,
    db: State<'_, DatabaseConnection>,
    rom_id: i64,
    source_id: i64,
    channel: Channel<DownloadProgress>,
    save_state_slot: Option<u32>,
    save_state_path: Option<String>,
) -> AppResult<()> {
    use sea_orm::{DatabaseBackend, FromQueryResult, Statement};

    #[derive(Debug, FromQueryResult)]
    struct RomDownloadInfo {
        file_name: String,
        file_size: Option<i64>,
        platform_id: i64,
        source_rom_id: String,
        source_type: String,
    }

    // 1. Get ROM info + source type (try exact source_id first, fall back to any source)
    let rom = RomDownloadInfo::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "SELECT r.file_name, r.file_size, r.platform_id, sr.source_rom_id, s.source_type
         FROM roms r
         JOIN source_roms sr ON sr.rom_id = r.id AND sr.source_id = ?
         JOIN sources s ON s.id = sr.source_id
         WHERE r.id = ?",
        [source_id.into(), rom_id.into()],
    ))
    .one(db.inner())
    .await?;

    let rom = if let Some(r) = rom {
        r
    } else {
        // Fallback: use any available source for this ROM
        RomDownloadInfo::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT r.file_name, r.file_size, r.platform_id, sr.source_rom_id, s.source_type
             FROM roms r
             JOIN source_roms sr ON sr.rom_id = r.id
             JOIN sources s ON s.id = sr.source_id
             WHERE r.id = ?
             LIMIT 1",
            [rom_id.into()],
        ))
        .one(db.inner())
        .await?
        .ok_or_else(|| AppError::Other("ROM not found in any source".to_string()))?
    };

    let RomDownloadInfo { file_name, file_size, platform_id, source_rom_id, source_type } = rom;

    // 2. Check core mapping exists
    #[derive(Debug, FromQueryResult)]
    struct CoreMappingRow {
        core_path: String,
        emulator_type: String,
    }

    let mapping = CoreMappingRow::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "SELECT core_path, emulator_type FROM core_mappings WHERE platform_id = ?",
        [platform_id.into()],
    ))
    .one(db.inner())
    .await?;

    let Some(CoreMappingRow { core_path, emulator_type }) = mapping else {
        return Err(AppError::Other(
            "No core mapped for this platform. Configure it in Settings.".to_string(),
        ));
    };

    // 3. Get emulator/RetroArch path
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;

    let is_retroarch = emulator_type == "retroarch";

    let ra_path = if is_retroarch {
        store
            .get("retroarch_path")
            .and_then(|v| v.as_str().map(std::string::ToString::to_string))
            .ok_or_else(|| {
                AppError::Other("RetroArch path not configured. Set it in Settings.".to_string())
            })?
    } else {
        // Get standalone emulator path from store
        let emu_paths: HashMap<String, String> = store
            .get("emulator_paths")
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        emu_paths.get(&emulator_type).cloned().ok_or_else(|| {
            AppError::Other(format!(
                "Emulator path not configured for {emulator_type}. Set it in Settings.",
            ))
        })?
    };

    // 4. Determine ROM path -- local sources use the file directly, remote sources download
    let rom_path = if source_type == "local" {
        let path = std::path::PathBuf::from(&source_rom_id);
        if !path.exists() {
            return Err(AppError::Other(format!(
                "ROM file not found: {source_rom_id}"
            )));
        }
        path
    } else {
        let cache_dir = directories::ProjectDirs::from("com", "romm-buddy", "romm-buddy")
            .map_or_else(|| std::path::PathBuf::from("rom_cache"), |p| p.cache_dir().join("rom_cache"));
        std::fs::create_dir_all(&cache_dir)?;

        let cached = cache_dir.join(&file_name);
        if !cached.exists() {
            let _ = channel.send(DownloadProgress::status(rom_id, "downloading"));

            // ROMM: authenticated download
            #[derive(Debug, FromQueryResult)]
            struct SourceCredRow {
                url: String,
                credentials: String,
            }
            let cred_row = SourceCredRow::find_by_statement(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "SELECT url, credentials FROM sources WHERE id = ?",
                [source_id.into()],
            ))
            .one(db.inner())
            .await?
            .ok_or_else(|| AppError::Other("Source not found".to_string()))?;
            let (base_url, credentials) = (cred_row.url, cred_row.credentials);

            let creds: std::collections::HashMap<String, String> =
                serde_json::from_str(&credentials).unwrap_or_default();
            let username = creds.get("username").cloned().unwrap_or_default();
            let password = creds.get("password").cloned().unwrap_or_default();

            let client = RommClient::new(base_url, username, password);
            #[allow(clippy::similar_names)]
            let romm_id: i64 = source_rom_id.parse().map_err(|_| {
                AppError::Other("Invalid source ROM ID".to_string())
            })?;

            let resp = client.download_rom(romm_id, &file_name).await?;

            let total_bytes = resp.content_length()
                .or_else(|| file_size.and_then(|s| u64::try_from(s).ok()))
                .unwrap_or(0);
            let mut downloaded: u64 = 0;

            // Download to a temp file, then rename atomically to avoid partial cached files
            let tmp_path = cache_dir.join(format!(".{file_name}.part"));
            let mut file = tokio::fs::File::create(&tmp_path).await?;
            let mut stream = resp.bytes_stream();

            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                #[allow(clippy::cast_possible_truncation)]
                {
                    downloaded += chunk.len() as u64;
                }
                tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
                let _ = channel.send(DownloadProgress::downloading(rom_id, downloaded, total_bytes));
            }
            file.flush().await?;
            file.sync_all().await?;
            drop(file);
            tokio::fs::rename(&tmp_path, &cached).await?;
        }
        cached
    };

    // 6. Update play stats (upsert — library row may not exist yet)
    {
        use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
        let _ = db.inner().execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "INSERT INTO library (rom_id, source_id, play_count, last_played_at)
             VALUES (?, ?, 1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(rom_id, source_id) DO UPDATE SET
                play_count = play_count + 1,
                last_played_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
            [rom_id.into(), source_id.into()],
        )).await;
    }

    // 7. Launch RetroArch
    let _ = channel.send(DownloadProgress::status(rom_id, "launching"));

    let rom_path_str = rom_path.to_string_lossy().to_string();

    if is_retroarch {
        log::info!(
            "Launching RetroArch: ra_path={ra_path}, core_path={core_path}, rom_path={rom_path_str}, source_type={source_type}",
        );

        // On macOS, .app binaries must be launched via `open` to work properly with LaunchServices.
        let status = if ra_path.contains(".app/") {
            let app_path = ra_path.split(".app/").next().unwrap_or(&ra_path).to_string() + ".app";
            log::info!("Launching via: open {app_path} --args -L {core_path} {rom_path_str}");
            let mut cmd = std::process::Command::new("open");
            cmd.arg(&app_path)
                .arg("--args")
                .arg("-L")
                .arg(&core_path)
                .arg(&rom_path_str);
            if let Some(slot) = save_state_slot {
                cmd.arg("-e").arg(slot.to_string());
            }
            cmd.status()
        } else {
            log::info!("Launching binary directly: {ra_path} -L {core_path} {rom_path_str}");
            let mut cmd = std::process::Command::new(&ra_path);
            cmd.arg("-L")
                .arg(&core_path)
                .arg(&rom_path_str);
            if let Some(slot) = save_state_slot {
                cmd.arg("-e").arg(slot.to_string());
            }
            cmd.spawn()
                .map(|_| ())
                .map_err(|e| AppError::Other(format!("Failed to launch RetroArch: {e}")))?;
            let _ = channel.send(DownloadProgress::status(rom_id, "done"));
            return Ok(());
        };

        match status {
            Ok(s) => {
                log::info!("open command exited with: {s}");
                let _ = channel.send(DownloadProgress::status(rom_id, "done"));
                Ok(())
            }
            Err(e) => Err(AppError::Other(format!("Failed to launch RetroArch: {e}"))),
        }
    } else {
        // Standalone emulator launch
        let mut args = build_emulator_args(&emulator_type, &rom_path_str);

        // Append save state args for standalone emulators that support it
        if let Some(ref ss_path) = save_state_path {
            match emulator_type.as_str() {
                "mgba" => {
                    args.insert(0, ss_path.clone());
                    args.insert(0, "--savestate".into());
                }
                "dolphin" => {
                    args.push("-s".into());
                    args.push(ss_path.clone());
                }
                "duckstation" => {
                    args.push("-statefile".into());
                    args.push(ss_path.clone());
                }
                "pcsx2" => {
                    args.push("-statefile".into());
                    args.push(ss_path.clone());
                }
                _ => {}
            }
        }

        log::info!(
            "Launching standalone emulator: type={emulator_type}, path={ra_path}, args={args:?}",
        );

        // On macOS, use `open` for .app bundles
        let status = if std::path::Path::new(&ra_path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("app"))
        {
            let mut cmd = std::process::Command::new("open");
            cmd.arg(&ra_path).arg("--args");
            for arg in &args {
                cmd.arg(arg);
            }
            cmd.status()
        } else {
            std::process::Command::new(&ra_path)
                .args(&args)
                .spawn()
                .map(|_| ())
                .map_err(|e| AppError::Other(format!("Failed to launch emulator: {e}")))?;
            let _ = channel.send(DownloadProgress::status(rom_id, "done"));
            return Ok(());
        };

        match status {
            Ok(s) => {
                log::info!("open command exited with: {s}");
                let _ = channel.send(DownloadProgress::status(rom_id, "done"));
                Ok(())
            }
            Err(e) => Err(AppError::Other(format!("Failed to launch emulator: {e}"))),
        }
    }
}

#[tauri::command]
pub async fn get_available_cores(retroarch_path: String) -> AppResult<Vec<CoreInfo>> {
    let arch = if cfg!(target_arch = "aarch64") { "arm64" } else { "x86_64" };
    let url = format!(
        "https://buildbot.libretro.com/nightly/apple/osx/{arch}/latest/",
    );

    let tmp_file = std::env::temp_dir().join(format!("romm-buddy-cores-{}.html", std::process::id()));
    let tmp_path = tmp_file.to_string_lossy().to_string();
    let status = tokio::process::Command::new("/usr/bin/curl")
        .args(["-s", "-L", "-o", &tmp_path, &url])
        .status()
        .await
        .map_err(|e| AppError::Other(format!("Failed to run curl: {e}")))?;
    if !status.success() {
        return Err(AppError::Other(format!("curl failed with status {status}")));
    }
    let html = tokio::fs::read_to_string(&tmp_file)
        .await
        .map_err(|e| AppError::Other(format!("Failed to read curl output: {e}")))?;
    let _ = tokio::fs::remove_file(&tmp_file).await;
    log::info!("Buildbot fetch: html_len={}", html.len());

    // Parse .dylib.zip links from the HTML listing (scoped to drop non-Send types before await)
    let available_names = {
        let document = scraper::Html::parse_document(&html);
        let selector = scraper::Selector::parse("a[href]").expect("hardcoded CSS selector");
        let mut names: Vec<String> = Vec::new();
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if let Some(stripped) = href.strip_suffix(".dylib.zip") {
                    // href may be an absolute path; extract just the filename
                    let name = stripped.rsplit('/').next().unwrap_or(stripped);
                    if !name.is_empty() {
                        names.push(name.to_string());
                    }
                }
            }
        }
        names
    };

    // Get installed core names to filter them out
    let installed: Vec<CoreInfo> = detect_cores(retroarch_path).await.unwrap_or_default();
    let installed_names: std::collections::HashSet<&str> = installed
        .iter()
        .map(|c| c.core_name.as_str())
        .collect();

    let info_dir = find_info_dir();

    let mut cores: Vec<CoreInfo> = available_names
        .into_iter()
        .filter(|name| !installed_names.contains(name.as_str()))
        .map(|name| {
            let display_name = info_dir.as_ref().and_then(|dir| {
                parse_display_name(&dir.join(format!("{name}.info")))
            });
            CoreInfo {
                core_name: name,
                core_path: String::new(),
                display_name,
            }
        })
        .collect();

    cores.sort_by(|a, b| a.core_name.cmp(&b.core_name));
    Ok(cores)
}

#[tauri::command]
pub async fn install_core(retroarch_path: String, core_name: String) -> AppResult<CoreInfo> {
    let arch = if cfg!(target_arch = "aarch64") { "arm64" } else { "x86_64" };
    let url = format!(
        "https://buildbot.libretro.com/nightly/apple/osx/{arch}/latest/{core_name}.dylib.zip",
    );

    let cores_dir = find_cores_dir(&retroarch_path).ok_or_else(|| {
        AppError::Other("Could not find RetroArch cores directory".to_string())
    })?;

    // Download the zip via curl (Cloudflare blocks reqwest)
    let tmp_file = std::env::temp_dir().join(format!("romm-buddy-{core_name}.dylib.zip"));
    let tmp_path = tmp_file.to_string_lossy().to_string();
    let status = tokio::process::Command::new("curl")
        .args(["-s", "-L", "-f", "-o", &tmp_path, &url])
        .status()
        .await
        .map_err(|e| AppError::Other(format!("Failed to run curl: {e}")))?;
    if !status.success() {
        return Err(AppError::Other(format!(
            "Failed to download core: curl exit {status}"
        )));
    }
    let bytes = tokio::fs::read(&tmp_file)
        .await
        .map_err(|e| AppError::Other(format!("Failed to read downloaded zip: {e}")))?;
    let _ = tokio::fs::remove_file(&tmp_file).await;

    // Extract .dylib from zip
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| AppError::Other(format!("Failed to open zip: {e}")))?;

    let mut extracted = false;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::Other(format!("Failed to read zip entry: {e}")))?;
        let name = file.name().to_string();
        if std::path::Path::new(&name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("dylib"))
        {
            let out_path = cores_dir.join(&name);
            let mut out_file = std::fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut out_file)?;
            extracted = true;
            break;
        }
    }

    if !extracted {
        return Err(AppError::Other(
            "No .dylib found in downloaded zip".to_string(),
        ));
    }

    let core_path = cores_dir
        .join(format!("{core_name}.dylib"))
        .to_string_lossy()
        .to_string();

    let display_name = find_info_dir().and_then(|dir| {
        parse_display_name(&dir.join(format!("{core_name}.info")))
    });

    Ok(CoreInfo {
        core_name,
        core_path,
        display_name,
    })
}

/// Key for the cancellation token map — avoids magic sentinel i64 values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CancelKey {
    Source(i64),
    Metadata,
    Verification,
}

/// Managed state for sync cancellation tokens.
pub struct CancelTokenMap(pub tokio::sync::Mutex<HashMap<CancelKey, CancellationToken>>);

// -- Metadata enrichment commands --

#[tauri::command]
pub async fn update_launchbox_db(
    db: State<'_, DatabaseConnection>,
    cancel_tokens: State<'_, CancelTokenMap>,
    channel: Channel<ScanProgress>,
) -> AppResult<()> {
    let cancel = CancellationToken::new();
    cancel_tokens.0.lock().await.insert(CancelKey::Metadata, cancel.clone());

    // Download and extract Metadata.xml
    let channel_clone = channel.clone();
    crate::metadata::launchbox::download_and_extract(move |progress| {
        let _ = channel_clone.send(progress);
    }, cancel.clone())
    .await?;

    // Import into SQLite tables
    let result = crate::metadata::launchbox::import_to_db(db.inner(), move |progress| {
        let _ = channel.send(progress);
    })
    .await;
    cancel_tokens.0.lock().await.remove(&CancelKey::Metadata);
    result
}

#[tauri::command]
pub async fn fetch_metadata(
    app: tauri::AppHandle,
    db: State<'_, DatabaseConnection>,
    cancel_tokens: State<'_, CancelTokenMap>,
    platform_id: Option<i64>,
    search: Option<String>,
    channel: Channel<ScanProgress>,
) -> AppResult<()> {
    let cancel = CancellationToken::new();
    cancel_tokens.0.lock().await.insert(CancelKey::Metadata, cancel.clone());

    // Read IGDB credentials and construct client if available
    let igdb_client = read_igdb_client_from_store(&app);

    // Read ScreenScraper credentials if available
    let ss_creds = read_ss_creds_from_store(&app);

    let result = crate::metadata::enrich_roms(
        platform_id,
        search.as_deref(),
        db.inner(),
        move |progress| {
            let _ = channel.send(progress);
        },
        cancel,
        igdb_client.as_ref(),
        ss_creds.as_ref(),
    )
    .await;

    cancel_tokens.0.lock().await.remove(&CancelKey::Metadata);
    result
}

#[tauri::command]
pub async fn cancel_metadata(
    cancel_tokens: State<'_, CancelTokenMap>,
) -> AppResult<()> {
    if let Some(token) = cancel_tokens.0.lock().await.get(&CancelKey::Metadata) {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
pub async fn has_launchbox_db(
    db: State<'_, DatabaseConnection>,
) -> AppResult<bool> {
    Ok(crate::metadata::launchbox::has_imported_db(db.inner()).await)
}

async fn compute_rom_hash_inner(
    db: &DatabaseConnection,
    rom_id: i64,
) -> AppResult<Option<String>> {
    use crate::entity::roms;
    use sea_orm::{ConnectionTrait, DatabaseBackend, EntityTrait, FromQueryResult, Statement};

    // Check if already computed
    let rom_model = roms::Entity::find_by_id(rom_id).one(db).await?;
    if let Some(ref rom) = rom_model {
        if let Some(ref h) = rom.hash_md5 {
            if !h.is_empty() {
                return Ok(Some(h.clone()));
            }
        }
    }

    // Get ROM info to determine how to access the file
    #[derive(Debug, FromQueryResult)]
    struct RomInfoRow {
        file_name: String,
        source_rom_id: String,
        source_type: String,
        source_id: i64,
    }
    let info = RomInfoRow::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "SELECT r.file_name, sr.source_rom_id, s.source_type, sr.source_id FROM roms r JOIN source_roms sr ON sr.rom_id = r.id JOIN sources s ON s.id = sr.source_id WHERE r.id = ?",
        [rom_id.into()],
    ))
    .one(db)
    .await?
    .ok_or_else(|| AppError::Other(format!("ROM {rom_id} not found")))?;

    let (file_name, source_rom_id, source_type, source_id) =
        (info.file_name, info.source_rom_id, info.source_type, info.source_id);

    if source_type == "local" {
        // Local: hash the file directly (extract from zip if needed)
        let path = std::path::PathBuf::from(&source_rom_id);
        if !path.exists() {
            return Err(AppError::Other("ROM file not found on disk".into()));
        }
        let hash = tokio::task::spawn_blocking(move || -> Result<String, String> {
            use md5::{Digest, Md5};
            let lower = path.to_string_lossy().to_lowercase();
            if lower.ends_with(".zip") {
                let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
                let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
                if archive.is_empty() {
                    return Err("Empty zip archive".into());
                }
                let mut inner = archive.by_index(0).map_err(|e| e.to_string())?;
                let mut hasher = Md5::new();
                std::io::copy(&mut inner, &mut hasher).map_err(|e| e.to_string())?;
                Ok(format!("{:x}", hasher.finalize()))
            } else {
                let mut file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
                let mut hasher = Md5::new();
                std::io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
                Ok(format!("{:x}", hasher.finalize()))
            }
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
        .map_err(|e| AppError::Other(format!("Failed to compute hash: {e}")))?;

        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "UPDATE roms SET hash_md5 = ? WHERE id = ?",
            [hash.clone().into(), rom_id.into()],
        ))
        .await?;
        return Ok(Some(hash));
    }

    // Remote: download to temp file, hash, delete
    let tmp_dir = std::env::temp_dir().join("romm-buddy-hash");
    std::fs::create_dir_all(&tmp_dir)?;
    let tmp_path = tmp_dir.join(&file_name);

    // ROMM: authenticated download
    #[derive(Debug, FromQueryResult)]
    struct SourceCredsRow {
        url: String,
        credentials: String,
    }
    let creds_row = SourceCredsRow::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "SELECT url, credentials FROM sources WHERE id = ?",
        [source_id.into()],
    ))
    .one(db)
    .await?
    .ok_or_else(|| AppError::Other(format!("Source {source_id} not found")))?;
    let (base_url, credentials) = (creds_row.url, creds_row.credentials);

    let creds: std::collections::HashMap<String, String> =
        serde_json::from_str(&credentials).unwrap_or_default();
    let username = creds.get("username").cloned().unwrap_or_default();
    let password = creds.get("password").cloned().unwrap_or_default();

    let client = RommClient::new(base_url, username, password);
    let romm_id: i64 = source_rom_id
        .parse()
        .map_err(|_| AppError::Other("Invalid source ROM ID".into()))?;

    let resp = client.download_rom(romm_id, &file_name).await?;

    if !resp.status().is_success() {
        return Err(AppError::Other(format!(
            "Download failed: {}",
            resp.status()
        )));
    }

    // Stream to temp file
    let mut file = tokio::fs::File::create(&tmp_path).await?;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
    }
    file.flush().await?;
    drop(file);

    // Compute hash — extract from zip/7z if needed (RA expects uncompressed ROM hash)
    let hash_path = tmp_path.clone();
    let hash = tokio::task::spawn_blocking(move || -> Result<String, String> {
        use md5::{Digest, Md5};

        let lower = hash_path.to_string_lossy().to_lowercase();
        if lower.ends_with(".zip") {
            // Extract first file from zip, hash that
            let file = std::fs::File::open(&hash_path).map_err(|e| e.to_string())?;
            let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
            if archive.is_empty() {
                return Err("Empty zip archive".into());
            }
            let mut inner = archive.by_index(0).map_err(|e| e.to_string())?;
            log::info!("[RA] Hashing inner zip entry: {}", inner.name());
            let mut hasher = Md5::new();
            std::io::copy(&mut inner, &mut hasher).map_err(|e| e.to_string())?;
            Ok(format!("{:x}", hasher.finalize()))
        } else {
            // Hash file directly
            let mut f = std::fs::File::open(&hash_path).map_err(|e| e.to_string())?;
            let mut hasher = Md5::new();
            std::io::copy(&mut f, &mut hasher).map_err(|e| e.to_string())?;
            Ok(format!("{:x}", hasher.finalize()))
        }
    })
    .await
    .map_err(|e| AppError::Other(e.to_string()))?
    .map_err(|e| AppError::Other(format!("Failed to compute hash: {e}")))?;

    // Delete temp file
    let _ = tokio::fs::remove_file(&tmp_path).await;

    // Store hash
    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "UPDATE roms SET hash_md5 = ? WHERE id = ?",
        [hash.clone().into(), rom_id.into()],
    ))
    .await?;

    Ok(Some(hash))
}

#[tauri::command]
pub async fn compute_rom_hash(
    db: State<'_, DatabaseConnection>,
    rom_id: i64,
) -> AppResult<Option<String>> {
    compute_rom_hash_inner(db.inner(), rom_id).await
}

#[tauri::command]
pub async fn enrich_single_rom(
    app: tauri::AppHandle,
    db: State<'_, DatabaseConnection>,
    rom_id: i64,
) -> AppResult<RomWithMeta> {
    let igdb_client = read_igdb_client_from_store(&app);
    let ss_creds = read_ss_creds_from_store(&app);
    crate::metadata::enrich_single_rom(rom_id, db.inner(), igdb_client.as_ref(), ss_creds.as_ref()).await?;

    // Return the updated ROM data
    fetch_rom_with_meta(db.inner(), rom_id).await
}

/// Fetch a single ROM with all metadata, cover, and screenshots.
async fn fetch_rom_with_meta(db: &DatabaseConnection, rom_id: i64) -> AppResult<RomWithMeta> {
    use crate::entity::artwork;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

    let q = format!(
        "{ROM_WITH_META_SELECT} LEFT JOIN metadata m ON m.rom_id = r.id
         LEFT JOIN artwork a ON a.rom_id = r.id AND a.art_type = 'cover'
         LEFT JOIN hasheous_cache hc ON hc.rom_id = r.id
         LEFT JOIN source_roms sr ON sr.rom_id = r.id
                 LEFT JOIN sources s ON s.id = sr.source_id
         WHERE r.id = ?
         GROUP BY r.id",
    );
    let rows = query_rom_rows(db, &q, vec![rom_id.into()]).await?;
    let row = rows.into_iter().next()
        .ok_or_else(|| crate::error::AppError::Other(format!("ROM {rom_id} not found")))?;
    let mut rom = row.into_rom_with_meta();

    // Fetch screenshot URLs separately (multiple per ROM)
    rom.screenshot_urls = artwork::Entity::find()
        .filter(artwork::Column::RomId.eq(rom_id))
        .filter(artwork::Column::ArtType.eq("screenshot"))
        .order_by_asc(artwork::Column::Id)
        .all(db)
        .await?
        .into_iter()
        .filter_map(|m| m.url)
        .collect();

    Ok(rom)
}

#[tauri::command]
pub async fn get_rom(
    db: State<'_, DatabaseConnection>,
    rom_id: i64,
) -> AppResult<RomWithMeta> {
    fetch_rom_with_meta(db.inner(), rom_id).await
}

#[tauri::command]
pub async fn get_rom_screenshots(
    db: State<'_, DatabaseConnection>,
    rom_id: i64,
) -> AppResult<Vec<String>> {
    use crate::entity::artwork;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

    let models = artwork::Entity::find()
        .filter(artwork::Column::RomId.eq(rom_id))
        .filter(artwork::Column::ArtType.eq("screenshot"))
        .order_by_asc(artwork::Column::Id)
        .all(db.inner())
        .await?;
    Ok(models.into_iter().filter_map(|m| m.url).collect())
}

#[tauri::command]
pub async fn get_ra_credentials(
    app: tauri::AppHandle,
) -> AppResult<Option<crate::models::RaCredentials>> {
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    let username = store
        .get("retroachievements_username")
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let api_key = store
        .get("retroachievements_api_key")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    match (username, api_key) {
        (Some(u), Some(k)) if !u.is_empty() && !k.is_empty() => {
            Ok(Some(crate::models::RaCredentials {
                username: u,
                api_key: k,
            }))
        }
        _ => Ok(None),
    }
}

#[tauri::command]
pub async fn set_ra_credentials(
    app: tauri::AppHandle,
    username: String,
    api_key: String,
) -> AppResult<()> {
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    store.set("retroachievements_username", serde_json::json!(username));
    store.set("retroachievements_api_key", serde_json::json!(api_key));
    store
        .save()
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn test_ra_connection(username: String, api_key: String) -> AppResult<RaTestResult> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();
    Ok(crate::retroachievements::test_connection(&client, &username, &api_key).await)
}

#[tauri::command]
pub async fn get_achievements(
    app: tauri::AppHandle,
    db: State<'_, DatabaseConnection>,
    rom_id: i64,
) -> AppResult<AchievementData> {
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    let username = store
        .get("retroachievements_username")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or_else(|| AppError::Other("RA username not configured".into()))?;
    let api_key = store
        .get("retroachievements_api_key")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or_else(|| AppError::Other("RA API key not configured".into()))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    use sea_orm::{ConnectionTrait, DatabaseBackend, FromQueryResult, Statement};

    // Try to get RA game ID from hasheous cache first
    let cached_id = {
        let result = db.inner()
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "SELECT retroachievements_game_id FROM hasheous_cache WHERE rom_id = ?",
                [rom_id.into()],
            ))
            .await?;
        result.and_then(|row| row.try_get_by_index::<Option<String>>(0).ok()).flatten()
    };

    let ra_game_id = if let Some(id) = cached_id {
        log::info!("[RA] Using cached RA game ID: {id} for rom {rom_id}");
        id
    } else {
        // Fallback: search RA's game list by ROM hash
        #[derive(Debug, FromQueryResult)]
        struct RomHashInfo {
            slug: String,
            hash_md5: Option<String>,
        }
        let rom_info = RomHashInfo::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT p.slug, r.hash_md5 FROM roms r JOIN platforms p ON p.id = r.platform_id WHERE r.id = ?",
            [rom_id.into()],
        ))
        .one(db.inner())
        .await?
        .ok_or_else(|| AppError::Other(format!("ROM {rom_id} not found")))?;

        let (platform_slug, md5) = (rom_info.slug, rom_info.hash_md5);
        log::info!("[RA] ROM {rom_id}: platform_slug={platform_slug}, has_md5={}", md5.is_some());

        // If ROM has no hash, compute it on-demand (downloads remote ROMs temporarily)
        let md5 = match md5 {
            Some(h) if !h.is_empty() => h,
            _ => {
                log::info!("[RA] ROM {rom_id}: computing hash on-demand...");
                compute_rom_hash_inner(db.inner(), rom_id)
                    .await?
                    .ok_or_else(|| {
                        AppError::Other(
                            "No RetroAchievements game found for this ROM".into(),
                        )
                    })?
            }
        };

        log::info!("[RA] ROM {rom_id}: md5={md5}, looking up RA game by hash for platform {platform_slug}...");

        let mut found_id = crate::retroachievements::find_game_id_by_hash(
            &client,
            &username,
            &api_key,
            &platform_slug,
            &md5,
        )
        .await;

        // If lookup failed, the stored hash might be from a zip file (pre-fix).
        // Clear it and recompute with zip-aware logic.
        if found_id.is_none() {
            log::info!("[RA] ROM {rom_id}: hash {md5} not found in RA, clearing and recomputing...");
            let _ = db.inner()
                .execute(Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    "UPDATE roms SET hash_md5 = NULL WHERE id = ?",
                    [rom_id.into()],
                ))
                .await;
            if let Ok(Some(new_md5)) = compute_rom_hash_inner(db.inner(), rom_id).await {
                if new_md5 != md5 {
                    log::info!("[RA] ROM {rom_id}: recomputed hash={new_md5} (was {md5}), retrying RA lookup...");
                    found_id = crate::retroachievements::find_game_id_by_hash(
                        &client,
                        &username,
                        &api_key,
                        &platform_slug,
                        &new_md5,
                    )
                    .await;
                }
            }
        }

        let found_id = found_id.ok_or_else(|| {
            AppError::Other("No RetroAchievements game found for this ROM".into())
        })?;

        log::info!("[RA] ROM {rom_id}: found RA game ID: {found_id}");

        // Cache the discovered RA game ID in hasheous_cache for next time
        let _ = db.inner()
            .execute(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "INSERT INTO hasheous_cache (rom_id, retroachievements_game_id)
                 VALUES (?, ?)
                 ON CONFLICT(rom_id) DO UPDATE SET retroachievements_game_id = excluded.retroachievements_game_id",
                [rom_id.into(), found_id.clone().into()],
            ))
            .await;

        found_id
    };

    crate::retroachievements::fetch_game_achievements(&client, &username, &api_key, &ra_game_id)
        .await
}

/// A source link for a ROM (returned by get_rom_sources).
#[derive(Debug, serde::Serialize, sea_orm::FromQueryResult)]
pub struct RomSource {
    pub source_id: i64,
    pub source_name: String,
    pub source_type: String,
    pub source_rom_id: Option<String>,
    pub source_url: Option<String>,
    pub file_name: Option<String>,
    pub hash_md5: Option<String>,
}

#[tauri::command]
pub async fn get_rom_sources(
    db: State<'_, DatabaseConnection>,
    rom_id: i64,
) -> AppResult<Vec<RomSource>> {
    use sea_orm::{DatabaseBackend, FromQueryResult, Statement};

    let rows = RomSource::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "SELECT sr.source_id, s.name as source_name, s.source_type, sr.source_rom_id, sr.source_url, sr.file_name, sr.hash_md5 FROM source_roms sr JOIN sources s ON s.id = sr.source_id WHERE sr.rom_id = ? ORDER BY s.name",
        [rom_id.into()],
    ))
    .all(db.inner())
    .await?;
    Ok(rows)
}

#[tauri::command]
pub async fn deduplicate_roms(db: State<'_, DatabaseConnection>) -> AppResult<u64> {
    crate::dedup::reconcile_duplicates(db.inner()).await
}

// ---------- DAT verification commands ----------

#[tauri::command]
pub async fn import_dat_file(
    db: State<'_, DatabaseConnection>,
    file_path: String,
    dat_type: String,
    platform_slug: String,
    channel: Channel<ScanProgress>,
) -> AppResult<i64> {
    let path = std::path::PathBuf::from(file_path);
    crate::metadata::dat::import_dat_file(
        db.inner(),
        &path,
        &dat_type,
        &platform_slug,
        move |p| { let _ = channel.send(p); },
    )
    .await
}

#[tauri::command]
pub async fn get_dat_files(
    db: State<'_, DatabaseConnection>,
) -> AppResult<Vec<crate::metadata::dat::DatFileInfo>> {
    use crate::entity::dat_files;
    use sea_orm::{EntityTrait, QueryOrder};

    let models = dat_files::Entity::find()
        .order_by_asc(dat_files::Column::PlatformSlug)
        .order_by_asc(dat_files::Column::DatType)
        .all(db.inner())
        .await?;

    Ok(models
        .into_iter()
        .map(|m| crate::metadata::dat::DatFileInfo {
            id: m.id,
            name: m.name,
            description: m.description,
            version: m.version,
            dat_type: m.dat_type,
            platform_slug: m.platform_slug,
            entry_count: m.entry_count,
            imported_at: m.imported_at,
        })
        .collect())
}

#[tauri::command]
pub async fn remove_dat_file(
    db: State<'_, DatabaseConnection>,
    dat_file_id: i64,
) -> AppResult<()> {
    use crate::entity::dat_files;
    use sea_orm::{EntityTrait, ModelTrait};

    if let Some(model) = dat_files::Entity::find_by_id(dat_file_id).one(db.inner()).await? {
        model.delete(db.inner()).await?;
    }
    Ok(())
}

#[derive(Debug, serde::Serialize)]
pub struct DatDetectResult {
    pub detected_slug: Option<String>,
    pub header_name: String,
}

#[tauri::command]
pub async fn detect_dat_platform(file_path: String) -> AppResult<DatDetectResult> {
    let path = std::path::PathBuf::from(file_path);
    let parsed = tokio::task::spawn_blocking(move || {
        crate::metadata::dat::parse_dat_file(&path)
    })
    .await
    .map_err(|e| AppError::Other(format!("Task join error: {e}")))?
    ?;
    let detected_slug = crate::metadata::dat::detect_platform_slug(&parsed.header.name);
    Ok(DatDetectResult {
        detected_slug,
        header_name: parsed.header.name,
    })
}

#[tauri::command]
pub async fn verify_library(
    db: State<'_, DatabaseConnection>,
    cancel_map: State<'_, CancelTokenMap>,
    platform_id: Option<i64>,
    channel: Channel<ScanProgress>,
) -> AppResult<crate::metadata::dat::VerificationStats> {
    let cancel = CancellationToken::new();
    {
        let mut map = cancel_map.0.lock().await;
        map.insert(CancelKey::Verification, cancel.clone());
    }
    let result = crate::metadata::dat::verify_roms(
        db.inner(),
        platform_id,
        move |p| { let _ = channel.send(p); },
        cancel,
    )
    .await;
    {
        let mut map = cancel_map.0.lock().await;
        map.remove(&CancelKey::Verification);
    }
    result
}

#[tauri::command]
pub async fn cancel_verification(
    cancel_map: State<'_, CancelTokenMap>,
) -> AppResult<()> {
    let map = cancel_map.0.lock().await;
    if let Some(token) = map.get(&CancelKey::Verification) {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
pub async fn get_verification_stats(
    db: State<'_, DatabaseConnection>,
    platform_id: Option<i64>,
) -> AppResult<crate::metadata::dat::VerificationStats> {
    crate::metadata::dat::get_verification_stats(db.inner(), platform_id).await
}

// ---------- IGDB credential commands ----------

/// Helper to read IGDB credentials from the store and construct an IgdbClient if available.
fn read_igdb_client_from_store(
    app: &tauri::AppHandle,
) -> Option<crate::metadata::igdb::IgdbClient> {
    let store = app.store("settings.json").ok()?;
    let client_id = store
        .get("igdb_client_id")
        .and_then(|v| v.as_str().map(|s| s.to_string()))?;
    let client_secret = store
        .get("igdb_client_secret")
        .and_then(|v| v.as_str().map(|s| s.to_string()))?;

    if client_id.is_empty() || client_secret.is_empty() {
        return None;
    }

    Some(crate::metadata::igdb::IgdbClient::new(
        client_id,
        client_secret,
    ))
}

#[tauri::command]
pub async fn get_igdb_credentials(
    app: tauri::AppHandle,
) -> AppResult<Option<crate::models::IgdbCredentials>> {
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    let client_id = store
        .get("igdb_client_id")
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let client_secret = store
        .get("igdb_client_secret")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    match (client_id, client_secret) {
        (Some(id), Some(secret)) if !id.is_empty() && !secret.is_empty() => {
            Ok(Some(crate::models::IgdbCredentials {
                client_id: id,
                client_secret: secret,
            }))
        }
        _ => Ok(None),
    }
}

#[tauri::command]
pub async fn set_igdb_credentials(
    app: tauri::AppHandle,
    client_id: String,
    client_secret: String,
) -> AppResult<()> {
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    store.set("igdb_client_id", serde_json::json!(client_id));
    store.set("igdb_client_secret", serde_json::json!(client_secret));
    store
        .save()
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn test_igdb_connection(
    client_id: String,
    client_secret: String,
) -> AppResult<IgdbTestResult> {
    let client = crate::metadata::igdb::IgdbClient::new(client_id, client_secret);
    client.test_connection().await
}

// ---------- ScreenScraper credential commands ----------

/// Helper to read ScreenScraper user credentials from the store.
fn read_ss_creds_from_store(
    app: &tauri::AppHandle,
) -> Option<crate::metadata::screenscraper::SsUserCredentials> {
    let store = app.store("settings.json").ok()?;
    let username = store
        .get("screenscraper_username")
        .and_then(|v| v.as_str().map(|s| s.to_string()))?;
    let password = store
        .get("screenscraper_password")
        .and_then(|v| v.as_str().map(|s| s.to_string()))?;

    if username.is_empty() || password.is_empty() {
        return None;
    }

    Some(crate::metadata::screenscraper::SsUserCredentials {
        username,
        password,
    })
}

#[tauri::command]
pub async fn get_ss_credentials(
    app: tauri::AppHandle,
) -> AppResult<Option<crate::models::SsCredentials>> {
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    let username = store
        .get("screenscraper_username")
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let password = store
        .get("screenscraper_password")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    match (username, password) {
        (Some(u), Some(p)) if !u.is_empty() && !p.is_empty() => {
            Ok(Some(crate::models::SsCredentials {
                username: u,
                password: p,
            }))
        }
        _ => Ok(None),
    }
}

#[tauri::command]
pub async fn set_ss_credentials(
    app: tauri::AppHandle,
    username: String,
    password: String,
) -> AppResult<()> {
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    store.set("screenscraper_username", serde_json::json!(username));
    store.set("screenscraper_password", serde_json::json!(password));
    store
        .save()
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn test_ss_connection(
    username: String,
    password: String,
) -> AppResult<SsTestResult> {
    let client = reqwest::Client::builder()
        .user_agent("romm-buddy/0.1")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();
    let creds = crate::metadata::screenscraper::SsUserCredentials {
        username,
        password,
    };
    crate::metadata::screenscraper::test_connection(&client, &creds).await
}

#[tauri::command]
pub async fn get_rom_saves(
    app: tauri::AppHandle,
    db: State<'_, DatabaseConnection>,
    rom_id: i64,
) -> AppResult<Vec<SaveFileInfo>> {
    use sea_orm::{ConnectionTrait, DatabaseBackend, FromQueryResult, Statement};

    // 1. Query ROM file_name, platform_id, and local file path (if local source)
    #[derive(Debug, FromQueryResult)]
    struct RomBasicInfo {
        file_name: String,
        platform_id: i64,
    }
    let rom_info = RomBasicInfo::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "SELECT file_name, platform_id FROM roms WHERE id = ?",
        [rom_id.into()],
    ))
    .one(db.inner())
    .await?
    .ok_or_else(|| AppError::Other(format!("ROM {rom_id} not found")))?;
    let (file_name, platform_id) = (rom_info.file_name, rom_info.platform_id);

    // Get the ROM's local file path (for "same directory as ROM" scanning)
    let rom_local_path: Option<String> = {
        let result = db.inner()
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "SELECT sr.source_rom_id FROM source_roms sr \
                 JOIN sources s ON s.id = sr.source_id \
                 WHERE sr.rom_id = ? AND s.source_type = 'local' \
                 LIMIT 1",
                [rom_id.into()],
            ))
            .await?;
        result.and_then(|row| row.try_get_by_index::<String>(0).ok())
    };

    // 2. Query emulator_type from core_mappings (default to "retroarch")
    let emulator_type = {
        let result = db.inner()
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "SELECT emulator_type FROM core_mappings WHERE platform_id = ? ORDER BY is_default DESC LIMIT 1",
                [platform_id.into()],
            ))
            .await?;
        result
            .and_then(|row| row.try_get_by_index::<String>(0).ok())
            .unwrap_or_else(|| "retroarch".to_string())
    };

    // 3. Get default save paths
    let defaults = saves::default_save_paths();
    let default_paths = defaults.get(emulator_type.as_str());

    let mut save_dirs: Vec<String> = default_paths
        .map(|p| p.save_dirs.clone())
        .unwrap_or_default();
    let mut state_dirs: Vec<String> = default_paths
        .map(|p| p.state_dirs.clone())
        .unwrap_or_default();

    // 4. Check user overrides from settings store
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    if let Some(overrides_val) = store.get("save_paths") {
        if let Ok(overrides) =
            serde_json::from_value::<HashMap<String, SavePathOverride>>(overrides_val)
        {
            if let Some(user_override) = overrides.get(&emulator_type) {
                // 5. Merge: user override dirs replace defaults if present
                if let Some(ref sd) = user_override.save_dir {
                    save_dirs = vec![sd.clone()];
                }
                if let Some(ref sd) = user_override.state_dir {
                    state_dirs = vec![sd.clone()];
                }
            }
        }
    }

    // 6. Also scan the ROM's own directory (emulators like mGBA save next to the ROM)
    if let Some(ref rom_path) = rom_local_path {
        if let Some(parent) = std::path::Path::new(rom_path).parent() {
            let parent_str = parent.to_string_lossy().into_owned();
            if !save_dirs.contains(&parent_str) {
                save_dirs.push(parent_str.clone());
            }
            if !state_dirs.contains(&parent_str) {
                state_dirs.push(parent_str);
            }
        }
    }

    // Also scan the ROM cache directory (for ROMM downloaded ROMs)
    if let Some(proj) = directories::ProjectDirs::from("com", "romm-buddy", "romm-buddy") {
        let cache_dir = proj.cache_dir().join("rom_cache").to_string_lossy().into_owned();
        if !save_dirs.contains(&cache_dir) {
            save_dirs.push(cache_dir.clone());
        }
        if !state_dirs.contains(&cache_dir) {
            state_dirs.push(cache_dir);
        }
    }

    // 7. Scan for saves
    Ok(saves::scan_for_saves(&file_name, &save_dirs, &state_dirs))
}

#[tauri::command]
pub async fn get_save_paths(
    app: tauri::AppHandle,
) -> AppResult<HashMap<String, SavePathOverride>> {
    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    let result = store
        .get("save_paths")
        .and_then(|v| serde_json::from_value::<HashMap<String, SavePathOverride>>(v).ok())
        .unwrap_or_default();
    Ok(result)
}

#[tauri::command]
pub async fn set_save_path(
    app: tauri::AppHandle,
    emulator_id: String,
    save_dir: Option<String>,
    state_dir: Option<String>,
) -> AppResult<()> {
    // Validate paths exist if provided
    if let Some(ref dir) = save_dir {
        if !std::path::Path::new(dir).is_dir() {
            return Err(AppError::Other(format!(
                "Save directory does not exist: {dir}"
            )));
        }
    }
    if let Some(ref dir) = state_dir {
        if !std::path::Path::new(dir).is_dir() {
            return Err(AppError::Other(format!(
                "State directory does not exist: {dir}"
            )));
        }
    }

    let store = app
        .store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;

    let mut overrides = store
        .get("save_paths")
        .and_then(|v| serde_json::from_value::<HashMap<String, SavePathOverride>>(v).ok())
        .unwrap_or_default();

    if save_dir.is_none() && state_dir.is_none() {
        // Remove the override entry entirely
        overrides.remove(&emulator_id);
    } else {
        overrides.insert(
            emulator_id,
            SavePathOverride {
                save_dir,
                state_dir,
            },
        );
    }

    store.set("save_paths", serde_json::json!(overrides));
    store
        .save()
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn delete_save_file(file_path: String) -> AppResult<()> {
    let path = std::path::PathBuf::from(&file_path);
    if !path.is_file() {
        return Err(AppError::Other(format!("File not found: {file_path}")));
    }
    tokio::fs::remove_file(&path)
        .await
        .map_err(|e| AppError::Other(format!("Failed to delete: {e}")))?;
    Ok(())
}

#[tauri::command]
pub async fn export_save_file(source_path: String, dest_path: String) -> AppResult<()> {
    let src = std::path::PathBuf::from(&source_path);
    if !src.is_file() {
        return Err(AppError::Other(format!(
            "Source file not found: {source_path}"
        )));
    }
    tokio::fs::copy(&src, &dest_path)
        .await
        .map_err(|e| AppError::Other(format!("Failed to export: {e}")))?;
    Ok(())
}

#[tauri::command]
pub async fn import_save_file(
    source_path: String,
    dest_dir: String,
    file_name: String,
) -> AppResult<()> {
    let src = std::path::PathBuf::from(&source_path);
    if !src.is_file() {
        return Err(AppError::Other(format!(
            "Source file not found: {source_path}"
        )));
    }
    let dest = std::path::Path::new(&dest_dir).join(&file_name);
    tokio::fs::copy(&src, &dest)
        .await
        .map_err(|e| AppError::Other(format!("Failed to import: {e}")))?;
    Ok(())
}

#[tauri::command]
pub async fn read_file_base64(file_path: String) -> AppResult<String> {
    use base64::Engine;
    let bytes = tokio::fs::read(&file_path)
        .await
        .map_err(|e| AppError::Other(format!("Failed to read file: {e}")))?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    ))
}

// ── Cache Management ──

#[tauri::command]
pub async fn get_cache_info(db: State<'_, DatabaseConnection>) -> AppResult<CacheInfo> {
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

    let cache_dir = rom_cache_dir();

    // Collect file info in a blocking task to avoid stalling the async runtime
    let file_entries: Vec<(String, u64)> = tokio::task::spawn_blocking(move || {
        let mut entries = Vec::new();
        if let Ok(dir_entries) = std::fs::read_dir(&cache_dir) {
            for entry in dir_entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                if file_name.starts_with('.') && file_name.ends_with(".part") {
                    continue;
                }
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                entries.push((file_name, size));
            }
        }
        entries
    })
    .await
    .map_err(|e| AppError::Other(format!("Task join error: {e}")))?;

    let mut files = Vec::new();
    let mut total_size: u64 = 0;

    for (file_name, size) in file_entries {
        total_size += size;

        let last_played: Option<String> = db.inner()
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "SELECT MAX(l.last_played_at) as last_played_at
                 FROM roms r JOIN library l ON l.rom_id = r.id
                 WHERE r.file_name = ?",
                [file_name.clone().into()],
            ))
            .await
            .ok()
            .flatten()
            .and_then(|row| row.try_get::<Option<String>>("", "last_played_at").ok())
            .flatten();

        files.push(CachedFile { file_name, size, last_played_at: last_played });
    }

    files.sort_by(|a, b| {
        b.last_played_at.cmp(&a.last_played_at)
            .then(b.size.cmp(&a.size))
    });

    Ok(CacheInfo { total_size, files })
}

#[tauri::command]
pub async fn clear_all_cache() -> AppResult<()> {
    let cache_dir = rom_cache_dir();
    tokio::task::spawn_blocking(move || {
        if let Ok(entries) = std::fs::read_dir(&cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    if name.starts_with('.') && name.ends_with(".part") {
                        continue;
                    }
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    })
    .await
    .map_err(|e| AppError::Other(format!("Task join error: {e}")))?;
    Ok(())
}

#[tauri::command]
pub async fn clear_cache_files(file_names: Vec<String>) -> AppResult<()> {
    let cache_dir = rom_cache_dir();
    tokio::task::spawn_blocking(move || {
        for file_name in &file_names {
            if file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
                continue;
            }
            let path = cache_dir.join(file_name);
            if path.is_file() {
                let _ = std::fs::remove_file(&path);
            }
        }
    })
    .await
    .map_err(|e| AppError::Other(format!("Task join error: {e}")))?;
    Ok(())
}

#[tauri::command]
pub async fn get_cache_eviction_days(app: tauri::AppHandle) -> AppResult<u32> {
    let store = app.store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(store.get("cache_eviction_days")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(7))
}

#[tauri::command]
pub async fn set_cache_eviction_days(app: tauri::AppHandle, days: u32) -> AppResult<()> {
    let store = app.store("settings.json")
        .map_err(|e| AppError::Other(e.to_string()))?;
    store.set("cache_eviction_days", serde_json::json!(days));
    store.save().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}
