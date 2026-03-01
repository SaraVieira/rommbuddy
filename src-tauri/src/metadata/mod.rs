pub mod dat;
pub mod hasheous;
pub mod igdb;
pub mod launchbox;
pub mod libretro_thumbnails;
pub mod screenscraper;

use std::collections::HashMap;
use std::path::PathBuf;

use md5::{Digest, Md5};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, FromQueryResult, Statement};
use tokio_util::sync::CancellationToken;

use crate::error::{AppError, AppResult};
use crate::models::ScanProgress;

#[derive(Debug, FromQueryResult)]
struct RomRow {
    id: i64,
    name: String,
    platform_slug: String,
    has_cover: i64,
    hash_md5: Option<String>,
    source_type: Option<crate::entity::sources::SourceType>,
    source_rom_id: Option<String>,
    screenscraper_id: Option<i64>,
}

/// Helper: look up igdb_game_id from hasheous_cache for a given rom_id.
async fn query_hasheous_igdb_id(db: &DatabaseConnection, rom_id: i64) -> Option<i64> {
    use crate::entity::hasheous_cache;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    hasheous_cache::Entity::find()
        .filter(hasheous_cache::Column::RomId.eq(rom_id))
        .one(db)
        .await
        .ok()
        .flatten()
        .and_then(|m| m.igdb_game_id)
}

/// Compute MD5 for a ROM file if not already stored.
/// For local ROMs, reads from `source_rom_id` path.
/// For downloaded ROMs, reads from the download cache.
async fn compute_md5_if_needed(db: &DatabaseConnection, rom: &RomRow) -> Option<String> {
    // Already computed
    if let Some(ref hash) = rom.hash_md5 {
        if !hash.is_empty() {
            return Some(hash.clone());
        }
    }

    // Determine file path
    let file_path = if rom.source_type == Some(crate::entity::sources::SourceType::Local) {
        rom.source_rom_id.as_ref().map(PathBuf::from)?
    } else {
        // For remote sources, check download cache
        let cache_dir = directories::ProjectDirs::from("com", "romm-buddy", "romm-buddy")
            .map(|p| p.cache_dir().join("rom_cache"))?;
        // We need the file_name from the roms table
        let file_name = db.query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT file_name FROM roms WHERE id = ?",
            [rom.id.into()],
        ))
        .await
        .ok()?
        .and_then(|r| r.try_get::<String>("", "file_name").ok())?;
        let path = cache_dir.join(&file_name);
        if !path.exists() {
            return None;
        }
        path
    };

    if !file_path.exists() {
        return None;
    }

    // Compute MD5 in a blocking task
    let rom_id = rom.id;
    let hash = tokio::task::spawn_blocking(move || -> Option<String> {
        let mut file = std::fs::File::open(&file_path).ok()?;
        let mut hasher = Md5::new();
        std::io::copy(&mut file, &mut hasher).ok()?;
        let result = hasher.finalize();
        Some(format!("{result:x}"))
    })
    .await
    .ok()??;

    // Store the hash
    if let Err(e) = db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "UPDATE roms SET hash_md5 = ? WHERE id = ?",
        [hash.clone().into(), rom_id.into()],
    ))
    .await
    {
        log::warn!("Failed to store MD5 hash for rom {rom_id}: {e}");
    }

    Some(hash)
}

const UNENRICHED_ROM_SELECT: &str = "SELECT r.id, r.name, p.slug as platform_slug,
        (SELECT COUNT(*) FROM artwork WHERE rom_id = r.id AND art_type = 'cover') as has_cover,
        r.hash_md5,
        (SELECT s2.source_type FROM source_roms sr2 JOIN sources s2 ON s2.id = sr2.source_id WHERE sr2.rom_id = r.id LIMIT 1) as source_type,
        (SELECT sr3.source_rom_id FROM source_roms sr3 JOIN sources s3 ON s3.id = sr3.source_id WHERE sr3.rom_id = r.id LIMIT 1) as source_rom_id,
        p.screenscraper_id
 FROM roms r
 JOIN platforms p ON p.id = r.platform_id
 LEFT JOIN metadata m ON m.rom_id = r.id
 LEFT JOIN hasheous_cache hc ON hc.rom_id = r.id";

/// Query ROMs that need enrichment, optionally filtered by platform and/or
/// search term (FTS match).
async fn fetch_unenriched_roms(
    db: &DatabaseConnection,
    platform_id: Option<i64>,
    search: Option<&str>,
) -> AppResult<Vec<RomRow>> {
    let search_query = search
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!("{}*", s.replace('"', "")));

    let has_search = search_query.is_some();

    let mut conditions = Vec::new();
    conditions.push("(has_cover = 0 OR m.metadata_fetched_at IS NULL OR hc.id IS NULL)".to_string());

    if platform_id.is_some() {
        conditions.push("r.platform_id = ?".to_string());
    }

    let fts_join = if has_search {
        conditions.push("roms_fts MATCH ?".to_string());
        " JOIN roms_fts ON roms_fts.rowid = r.id"
    } else {
        ""
    };

    let where_clause = conditions.join(" AND ");
    let sql = format!(
        "{UNENRICHED_ROM_SELECT}{fts_join} WHERE {where_clause} ORDER BY r.name"
    );

    let mut values: Vec<sea_orm::Value> = Vec::new();
    if let Some(pid) = platform_id {
        values.push(pid.into());
    }
    if let Some(ref fts) = search_query {
        values.push(fts.clone().into());
    }

    let stmt = Statement::from_sql_and_values(DatabaseBackend::Sqlite, &sql, values);
    Ok(RomRow::find_by_statement(stmt).all(db).await?)
}

/// Context shared by the enrichment pipeline.
struct EnrichContext<'a> {
    db: &'a DatabaseConnection,
    http_client: &'a reqwest::Client,
    igdb_client: Option<&'a igdb::IgdbClient>,
    ss_creds: Option<&'a screenscraper::SsUserCredentials>,
    has_launchbox: bool,
    last_ss_request: tokio::sync::Mutex<std::time::Instant>,
}

/// Options that differ between batch and single-ROM enrichment.
struct EnrichOptions {
    /// Pre-fetched IGDB data (batch optimization). None for single-ROM.
    igdb_prefetch: Option<igdb::IgdbGameData>,
    /// Whether to clear caches before enriching (true for single-ROM re-enrich).
    force_refresh: bool,
}

/// Insert artwork with dedup (ON CONFLICT DO NOTHING).
async fn insert_artwork(db: &DatabaseConnection, rom_id: i64, art_type: &str, url: &str) {
    if let Err(e) = db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO artwork (rom_id, art_type, url) VALUES (?, ?, ?) ON CONFLICT(rom_id, art_type, url) DO NOTHING",
        [rom_id.into(), art_type.into(), url.into()],
    ))
    .await
    {
        log::warn!("Failed to insert {art_type} artwork for rom {rom_id}: {e}");
    }
}

/// Unified per-ROM enrichment pipeline used by both `enrich_roms` and `enrich_single_rom`.
async fn enrich_one_rom(
    ctx: &EnrichContext<'_>,
    rom: &RomRow,
    opts: &EnrichOptions,
) -> AppResult<()> {
    let db = ctx.db;

    // Step 1: Compute hash if missing
    let md5 = compute_md5_if_needed(db, rom).await;

    // Step 2: Hasheous lookup
    let hasheous_result = if opts.force_refresh {
        // Single-ROM re-enrich: always fetch fresh from API
        if let Some(ref hash) = md5 {
            if let Some(result) = hasheous::lookup_by_md5(ctx.http_client, hash).await {
                hasheous::save_to_cache(db, rom.id, &result).await;
                Some(result)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        // Batch: check cache first, then API
        let cached = hasheous::get_cached(db, rom.id).await;
        match cached {
            Some(c) => Some(c),
            None => {
                if let Some(ref hash) = md5 {
                    if let Some(result) = hasheous::lookup_by_md5(ctx.http_client, hash).await {
                        hasheous::save_to_cache(db, rom.id, &result).await;
                        Some(result)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    };

    let hasheous_name = hasheous_result.as_ref().map(|r| r.name.as_str());

    // Upsert metadata from Hasheous
    if let Some(ref result) = hasheous_result {
        let genres_json =
            serde_json::to_string(&result.genres).unwrap_or_else(|_| "[]".to_string());
        if let Err(e) = db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "INSERT INTO metadata (rom_id, description, publisher, genres, release_date, metadata_fetched_at)
             VALUES (?, ?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(rom_id) DO UPDATE SET
               description = COALESCE(excluded.description, metadata.description),
               publisher = COALESCE(excluded.publisher, metadata.publisher),
               genres = CASE WHEN excluded.genres != '[]' THEN excluded.genres ELSE metadata.genres END,
               release_date = COALESCE(excluded.release_date, metadata.release_date),
               metadata_fetched_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
            [
                rom.id.into(),
                result.description.clone().into(),
                result.publisher.clone().into(),
                genres_json.into(),
                result.year.clone().into(),
            ],
        ))
        .await
        {
            log::warn!("Failed to upsert Hasheous metadata for rom {}: {e}", rom.id);
        }
    }

    // Step 3: IGDB enrichment
    if let Some(client) = ctx.igdb_client {
        let igdb_data = if let Some(ref prefetched) = opts.igdb_prefetch {
            Some(prefetched.clone())
        } else {
            // Try hasheous IGDB ID first, then name search
            let igdb_game_id = query_hasheous_igdb_id(db, rom.id).await;
            if let Some(igdb_id) = igdb_game_id {
                match client.fetch_games_by_ids(&[igdb_id]).await {
                    Ok(games) => games.into_iter().next(),
                    Err(e) => {
                        log::warn!("IGDB fetch failed for igdb_id {igdb_id}: {e}");
                        None
                    }
                }
            } else {
                let search_name = hasheous_name.unwrap_or(&rom.name);
                match client.search_game(search_name).await {
                    Ok(result) => result,
                    Err(e) => {
                        log::warn!("IGDB search failed for rom {}: {e}", rom.id);
                        None
                    }
                }
            }
        };

        if let Some(ref game) = igdb_data {
            apply_igdb_data(db, rom.id, game).await;
        }
    }

    // Step 4: LaunchBox lookup
    let lb_game = if ctx.has_launchbox {
        let lookup_name = hasheous_name.unwrap_or(&rom.name);
        launchbox::find_by_name(db, lookup_name, &rom.platform_slug).await
    } else {
        None
    };

    if let Some(ref lb_game) = lb_game {
        if let Err(e) = db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "INSERT INTO metadata (rom_id, description, developer, publisher, genres, release_date, rating, metadata_fetched_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(rom_id) DO UPDATE SET
               description = COALESCE(metadata.description, excluded.description),
               developer = COALESCE(excluded.developer, metadata.developer),
               publisher = COALESCE(metadata.publisher, excluded.publisher),
               genres = CASE WHEN metadata.genres = '[]' OR metadata.genres IS NULL THEN excluded.genres ELSE metadata.genres END,
               release_date = COALESCE(metadata.release_date, excluded.release_date),
               rating = COALESCE(excluded.rating, metadata.rating),
               metadata_fetched_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
            [
                rom.id.into(),
                lb_game.overview.clone().into(),
                lb_game.developer.clone().into(),
                lb_game.publisher.clone().into(),
                lb_game.genres.clone().into(),
                lb_game.release_date.clone().into(),
                lb_game.community_rating.into(),
            ],
        ))
        .await
        {
            log::warn!("Failed to upsert LaunchBox metadata for rom {}: {e}", rom.id);
        }

        if rom.has_cover == 0 {
            if let Some(url) = launchbox::get_image_url(db, &lb_game.database_id).await {
                insert_artwork(db, rom.id, "cover", &url).await;
            }
        }
    }

    // Step 5: ScreenScraper enrichment
    if let Some(ss_system_id) = rom.screenscraper_id {
        let should_lookup = if opts.force_refresh {
            // Clear cache on re-enrich
            let _ = db.execute(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "DELETE FROM screenscraper_cache WHERE rom_id = ?",
                [rom.id.into()],
            ))
            .await;
            true
        } else {
            !screenscraper::is_cached(db, rom.id).await
        };

        if should_lookup {
            match screenscraper::lookup_game(
                ctx.http_client,
                ctx.ss_creds,
                md5.as_deref(),
                &rom.name,
                ss_system_id,
                &ctx.last_ss_request,
            )
            .await
            {
                Ok(Some(ss_data)) => {
                    screenscraper::save_to_cache(
                        db,
                        rom.id,
                        ss_data.game_id,
                        &serde_json::to_string(&ss_data.game_id).unwrap_or_default(),
                    )
                    .await;
                    apply_screenscraper_metadata(db, rom.id, &ss_data).await;
                    apply_screenscraper_artwork(db, rom.id, &ss_data.media).await;
                }
                Ok(None) => {
                    screenscraper::save_to_cache(db, rom.id, None, "").await;
                }
                Err(e) => {
                    log::warn!("ScreenScraper lookup failed for rom {}: {e}", rom.id);
                }
            }
        }
    }

    // Step 6: libretro thumbnail (if still no cover)
    let current_has_cover = if rom.has_cover == 0 {
        let result = db.query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT COUNT(*) as cnt FROM artwork WHERE rom_id = ? AND art_type = 'cover'",
            [rom.id.into()],
        ))
        .await;
        result
            .ok()
            .flatten()
            .and_then(|r| r.try_get::<i64>("", "cnt").ok())
            .unwrap_or(0)
            > 0
    } else {
        true
    };

    if !current_has_cover {
        let name = hasheous_name.unwrap_or(&rom.name);
        if let Some(url) = libretro_thumbnails::build_thumbnail_url(&rom.platform_slug, name) {
            let exists = ctx.http_client
                .head(&url)
                .send()
                .await
                .is_ok_and(|r| r.status().is_success());
            if exists {
                insert_artwork(db, rom.id, "cover", &url).await;
            }
        }
    }

    // Step 7: Screenshot art — collect from all sources (libretro + LaunchBox)
    if opts.force_refresh {
        // Clear existing screenshots on re-enrich so we get fresh data
        let _ = db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "DELETE FROM artwork WHERE rom_id = ? AND art_type = 'screenshot'",
            [rom.id.into()],
        ))
        .await;
    }

    let snap_name = hasheous_name.unwrap_or(&rom.name);

    // libretro Named_Snaps + Named_Titles — fire HEAD requests concurrently
    let snap_url = libretro_thumbnails::build_snapshot_url(&rom.platform_slug, snap_name);
    let title_url = libretro_thumbnails::build_title_url(&rom.platform_slug, snap_name);

    let snap_future = async {
        if let Some(url) = &snap_url {
            ctx.http_client.head(url).send().await.is_ok_and(|r| r.status().is_success())
        } else {
            false
        }
    };
    let title_future = async {
        if let Some(url) = &title_url {
            ctx.http_client.head(url).send().await.is_ok_and(|r| r.status().is_success())
        } else {
            false
        }
    };

    let (snap_exists, title_exists) = tokio::join!(snap_future, title_future);

    if snap_exists {
        if let Some(url) = &snap_url {
            insert_artwork(db, rom.id, "screenshot", url).await;
        }
    }
    if title_exists {
        if let Some(url) = &title_url {
            insert_artwork(db, rom.id, "screenshot", url).await;
        }
    }

    // LaunchBox screenshots
    if let Some(ref lb_game) = lb_game {
        let urls = launchbox::get_screenshot_urls(db, &lb_game.database_id).await;
        for url in urls {
            insert_artwork(db, rom.id, "screenshot", &url).await;
        }
    }

    // Mark as enriched
    if let Err(e) = db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO metadata (rom_id, metadata_fetched_at)
         VALUES (?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         ON CONFLICT(rom_id) DO UPDATE SET
           metadata_fetched_at = COALESCE(metadata.metadata_fetched_at, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        [rom.id.into()],
    ))
    .await
    {
        log::warn!("Failed to mark rom {} as enriched: {e}", rom.id);
    }

    Ok(())
}

/// Enrich ROMs with metadata using the hash-first pipeline:
/// 1. Compute MD5 hash
/// 2. Hasheous API lookup (cached)
/// 3. IGDB enrichment (if client provided)
/// 4. `LaunchBox` SQL lookup using verified name
/// 5. ScreenScraper enrichment
/// 6. libretro-thumbnails cover art + screenshots
pub async fn enrich_roms(
    platform_id: Option<i64>,
    search: Option<&str>,
    db: &DatabaseConnection,
    on_progress: impl Fn(ScanProgress) + Send,
    cancel: CancellationToken,
    igdb_client: Option<&igdb::IgdbClient>,
    ss_creds: Option<&screenscraper::SsUserCredentials>,
) -> AppResult<()> {
    let roms = fetch_unenriched_roms(db, platform_id, search).await?;

    #[allow(clippy::cast_possible_truncation)]
    let total = roms.len() as u64;
    if total == 0 {
        on_progress(ScanProgress {
            source_id: -1,
            total: 0,
            current: 0,
            current_item: "All ROMs already enriched.".to_string(),
        });
        return Ok(());
    }

    let http_client = reqwest::Client::builder()
        .user_agent("romm-buddy/0.1")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    let has_launchbox = launchbox::has_imported_db(db).await;

    let ctx = EnrichContext {
        db,
        http_client: &http_client,
        igdb_client,
        ss_creds,
        has_launchbox,
        last_ss_request: tokio::sync::Mutex::new(std::time::Instant::now() - std::time::Duration::from_secs(2)),
    };

    // IGDB batch optimization: pre-collect all IGDB IDs from hasheous_cache,
    // batch-fetch in chunks of 10, build a HashMap for O(1) lookup during the loop
    let mut igdb_batch: HashMap<i64, igdb::IgdbGameData> = HashMap::new();
    if let Some(client) = igdb_client {
        let mut igdb_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
        for rom in &roms {
            if let Some(igdb_id) = query_hasheous_igdb_id(db, rom.id).await {
                igdb_ids.insert(igdb_id);
            }
        }

        let all_igdb_ids: Vec<i64> = igdb_ids.into_iter().collect();
        for chunk in all_igdb_ids.chunks(10) {
            if cancel.is_cancelled() {
                return Ok(());
            }
            match client.fetch_games_by_ids(chunk).await {
                Ok(games) => {
                    for game in games {
                        igdb_batch.insert(game.id, game);
                    }
                }
                Err(e) => {
                    log::warn!("IGDB batch fetch failed: {e}");
                }
            }
        }
    }

    for (i, rom) in roms.iter().enumerate() {
        if cancel.is_cancelled() {
            return Ok(());
        }

        #[allow(clippy::cast_possible_truncation)]
        let current = (i + 1) as u64;
        on_progress(ScanProgress {
            source_id: -1,
            total,
            current,
            current_item: rom.name.clone(),
        });

        // Look up pre-fetched IGDB data for this ROM
        let igdb_prefetch = query_hasheous_igdb_id(db, rom.id)
            .await
            .and_then(|igdb_id| igdb_batch.get(&igdb_id).cloned());

        let opts = EnrichOptions {
            igdb_prefetch,
            force_refresh: false,
        };

        enrich_one_rom(&ctx, rom, &opts).await?;
    }

    Ok(())
}

/// Enrich a single ROM by ID — runs the full enrichment pipeline.
/// Clears existing caches first so fresh data is fetched.
pub async fn enrich_single_rom(
    rom_id: i64,
    db: &DatabaseConnection,
    igdb_client: Option<&igdb::IgdbClient>,
    ss_creds: Option<&screenscraper::SsUserCredentials>,
) -> AppResult<()> {
    let rom = RomRow::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        format!("{UNENRICHED_ROM_SELECT} WHERE r.id = ?"),
        [rom_id.into()],
    ))
    .one(db)
    .await?
    .ok_or_else(|| AppError::Other(format!("ROM {rom_id} not found")))?;

    // Clear existing hasheous cache so we re-fetch
    let _ = db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "DELETE FROM hasheous_cache WHERE rom_id = ?",
        [rom_id.into()],
    ))
    .await;

    let http_client = reqwest::Client::builder()
        .user_agent("romm-buddy/0.1")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    let has_launchbox = launchbox::has_imported_db(db).await;

    let ctx = EnrichContext {
        db,
        http_client: &http_client,
        igdb_client,
        ss_creds,
        has_launchbox,
        last_ss_request: tokio::sync::Mutex::new(std::time::Instant::now() - std::time::Duration::from_secs(2)),
    };

    let opts = EnrichOptions {
        igdb_prefetch: None,
        force_refresh: true,
    };

    enrich_one_rom(&ctx, &rom, &opts).await
}

/// Apply IGDB game data to database: insert into igdb_cache, update metadata, save artwork.
async fn apply_igdb_data(db: &DatabaseConnection, rom_id: i64, game: &igdb::IgdbGameData) {
    // Insert into igdb_cache
    let genres_json = serde_json::to_string(&game.genre_names()).unwrap_or_else(|_| "[]".into());
    let themes_json = serde_json::to_string(&game.theme_names()).unwrap_or_else(|_| "[]".into());
    let game_modes_json =
        serde_json::to_string(&game.game_mode_names()).unwrap_or_else(|_| "[]".into());
    let player_perspectives_json =
        serde_json::to_string(&game.player_perspective_names()).unwrap_or_else(|_| "[]".into());
    let screenshot_ids_json =
        serde_json::to_string(&game.screenshot_image_ids()).unwrap_or_else(|_| "[]".into());
    let raw_response = serde_json::to_string(game).unwrap_or_default();

    if let Err(e) = db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO igdb_cache (rom_id, igdb_id, name, summary, storyline, aggregated_rating,
         first_release_date, genres, themes, game_modes, player_perspectives, developer, publisher,
         cover_image_id, screenshot_image_ids, franchise_name, raw_response)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(rom_id) DO UPDATE SET
           igdb_id = excluded.igdb_id,
           name = excluded.name,
           summary = excluded.summary,
           storyline = excluded.storyline,
           aggregated_rating = excluded.aggregated_rating,
           first_release_date = excluded.first_release_date,
           genres = excluded.genres,
           themes = excluded.themes,
           game_modes = excluded.game_modes,
           player_perspectives = excluded.player_perspectives,
           developer = excluded.developer,
           publisher = excluded.publisher,
           cover_image_id = excluded.cover_image_id,
           screenshot_image_ids = excluded.screenshot_image_ids,
           franchise_name = excluded.franchise_name,
           raw_response = excluded.raw_response,
           fetched_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
        [
            rom_id.into(),
            game.id.into(),
            game.name.clone().into(),
            game.summary.clone().into(),
            game.storyline.clone().into(),
            game.aggregated_rating.into(),
            game.first_release_date_string().into(),
            genres_json.clone().into(),
            themes_json.clone().into(),
            game_modes_json.into(),
            player_perspectives_json.into(),
            game.developer().into(),
            game.publisher().into(),
            game.cover_image_id().into(),
            screenshot_ids_json.into(),
            game.franchise_name().into(),
            raw_response.into(),
        ],
    ))
    .await
    {
        log::warn!("Failed to insert IGDB cache for rom {rom_id}: {e}");
    }

    // Update metadata table — IGDB overrides description, rating, genres, themes, developer, publisher
    let description = game.description();
    let rating = game.aggregated_rating.map(|r| r / 10.0); // IGDB is 0-100, normalize to 0-10
    let release_date = game.first_release_date_string();

    if let Err(e) = db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO metadata (rom_id, description, developer, publisher, genres, themes, rating, release_date, igdb_id, metadata_fetched_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         ON CONFLICT(rom_id) DO UPDATE SET
           description = COALESCE(excluded.description, metadata.description),
           developer = COALESCE(excluded.developer, metadata.developer),
           publisher = COALESCE(excluded.publisher, metadata.publisher),
           genres = CASE WHEN excluded.genres != '[]' THEN excluded.genres ELSE metadata.genres END,
           themes = CASE WHEN excluded.themes != '[]' THEN excluded.themes ELSE metadata.themes END,
           rating = COALESCE(excluded.rating, metadata.rating),
           release_date = COALESCE(excluded.release_date, metadata.release_date),
           igdb_id = COALESCE(excluded.igdb_id, metadata.igdb_id),
           metadata_fetched_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
        [
            rom_id.into(),
            description.into(),
            game.developer().into(),
            game.publisher().into(),
            genres_json.into(),
            themes_json.into(),
            rating.into(),
            release_date.into(),
            game.id.into(),
        ],
    ))
    .await
    {
        log::warn!("Failed to upsert IGDB metadata for rom {rom_id}: {e}");
    }

    // Save IGDB cover art
    if let Some(url) = game.cover_url() {
        insert_artwork(db, rom_id, "cover", &url).await;
    }

    // Save IGDB screenshots
    for url in game.screenshot_urls() {
        insert_artwork(db, rom_id, "screenshot", &url).await;
    }
}

/// Apply ScreenScraper metadata to database (only fill NULLs).
async fn apply_screenscraper_metadata(
    db: &DatabaseConnection,
    rom_id: i64,
    data: &screenscraper::SsGameData,
) {
    let genres_json = data
        .genre
        .as_ref()
        .map(|g| {
            let genres: Vec<&str> = g.split(", ").collect();
            serde_json::to_string(&genres).unwrap_or_else(|_| "[]".to_string())
        })
        .unwrap_or_else(|| "[]".to_string());

    if let Err(e) = db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO metadata (rom_id, description, developer, publisher, genres, release_date, rating, metadata_fetched_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         ON CONFLICT(rom_id) DO UPDATE SET
           description = COALESCE(metadata.description, excluded.description),
           developer = COALESCE(metadata.developer, excluded.developer),
           publisher = COALESCE(metadata.publisher, excluded.publisher),
           genres = CASE WHEN metadata.genres = '[]' OR metadata.genres IS NULL THEN excluded.genres ELSE metadata.genres END,
           release_date = COALESCE(metadata.release_date, excluded.release_date),
           rating = COALESCE(metadata.rating, excluded.rating),
           metadata_fetched_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
        [
            rom_id.into(),
            data.synopsis.clone().into(),
            data.developer.clone().into(),
            data.publisher.clone().into(),
            genres_json.into(),
            data.release_date.clone().into(),
            data.rating.into(),
        ],
    ))
    .await
    {
        log::warn!("Failed to upsert ScreenScraper metadata for rom {rom_id}: {e}");
    }
}

/// Apply ScreenScraper artwork (always append with ON CONFLICT DO NOTHING).
async fn apply_screenscraper_artwork(
    db: &DatabaseConnection,
    rom_id: i64,
    media: &[screenscraper::SsMedia],
) {
    for item in media {
        insert_artwork(db, rom_id, &item.media_type, &item.url).await;
    }
}
