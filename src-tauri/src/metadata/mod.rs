pub mod dat;
pub mod hasheous;
pub mod igdb;
pub mod launchbox;
pub mod libretro_thumbnails;
pub mod screenscraper;

use std::collections::HashMap;
use std::path::PathBuf;

use md5::{Digest, Md5};
use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

use crate::error::{AppError, AppResult};
use crate::models::ScanProgress;

#[derive(sqlx::FromRow)]
struct RomRow {
    id: i64,
    name: String,
    platform_slug: String,
    has_cover: i64,
    hash_md5: Option<String>,
    source_type: Option<String>,
    source_rom_id: Option<String>,
    screenscraper_id: Option<i64>,
}

/// Compute MD5 for a ROM file if not already stored.
/// For local ROMs, reads from `source_rom_id` path.
/// For downloaded ROMs, reads from the download cache.
async fn compute_md5_if_needed(pool: &SqlitePool, rom: &RomRow) -> Option<String> {
    // Already computed
    if let Some(ref hash) = rom.hash_md5 {
        if !hash.is_empty() {
            return Some(hash.clone());
        }
    }

    // Determine file path
    let file_path = if rom.source_type.as_deref() == Some("local") {
        rom.source_rom_id.as_ref().map(PathBuf::from)?
    } else {
        // For remote sources, check download cache
        let cache_dir = directories::ProjectDirs::from("com", "romm-buddy", "romm-buddy")
            .map(|p| p.cache_dir().join("rom_cache"))?;
        // We need the file_name from the roms table
        let file_name = sqlx::query_scalar::<_, String>(
            "SELECT file_name FROM roms WHERE id = ?",
        )
        .bind(rom.id)
        .fetch_optional(pool)
        .await
        .ok()??;
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
    if let Err(e) = sqlx::query("UPDATE roms SET hash_md5 = ? WHERE id = ?")
        .bind(&hash)
        .bind(rom_id)
        .execute(pool)
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
    pool: &SqlitePool,
    platform_id: Option<i64>,
    search: Option<&str>,
) -> AppResult<Vec<RomRow>> {
    let search_query = search
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!("{}*", s.replace('"', "")));

    let has_search = search_query.is_some();
    let has_platform = platform_id.is_some();

    let mut conditions = Vec::new();
    conditions.push("(has_cover = 0 OR m.metadata_fetched_at IS NULL OR hc.id IS NULL)".to_string());

    if has_platform {
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

    let mut query = sqlx::query_as::<_, RomRow>(&sql);
    if let Some(pid) = platform_id {
        query = query.bind(pid);
    }
    if let Some(ref fts) = search_query {
        query = query.bind(fts);
    }

    Ok(query.fetch_all(pool).await?)
}

/// Enrich ROMs with metadata using the hash-first pipeline:
/// 1. Compute MD5 hash
/// 2. Hasheous API lookup (cached)
/// 3. IGDB enrichment (if client provided)
/// 4. `LaunchBox` SQL lookup using verified name
/// 5. libretro-thumbnails cover art
pub async fn enrich_roms(
    platform_id: Option<i64>,
    search: Option<&str>,
    pool: &SqlitePool,
    on_progress: impl Fn(ScanProgress) + Send,
    cancel: CancellationToken,
    igdb_client: Option<&igdb::IgdbClient>,
    ss_creds: Option<&screenscraper::SsUserCredentials>,
) -> AppResult<()> {
    // 1. Query ROMs that haven't been enriched yet
    let roms = fetch_unenriched_roms(pool, platform_id, search).await?;

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

    let has_launchbox = launchbox::has_imported_db(pool).await;

    let http_client = reqwest::Client::builder()
        .user_agent("romm-buddy/0.1")
        .build()
        .unwrap_or_default();

    // IGDB batch optimization: pre-collect all IGDB IDs from hasheous_cache,
    // batch-fetch in chunks of 10, build a HashMap for O(1) lookup during the loop
    let mut igdb_batch: HashMap<i64, igdb::IgdbGameData> = HashMap::new();
    if let Some(client) = igdb_client {
        // Collect all (rom_id, igdb_game_id) pairs for the ROMs we're enriching
        let rom_ids: Vec<i64> = roms.iter().map(|r| r.id).collect();
        let mut igdb_id_to_rom_ids: HashMap<i64, Vec<i64>> = HashMap::new();

        for rom in &roms {
            if let Ok(Some(Some(igdb_id))) = sqlx::query_scalar::<_, Option<i64>>(
                "SELECT igdb_game_id FROM hasheous_cache WHERE rom_id = ? AND igdb_game_id IS NOT NULL",
            )
            .bind(rom.id)
            .fetch_optional(pool)
            .await
            {
                igdb_id_to_rom_ids
                    .entry(igdb_id)
                    .or_default()
                    .push(rom.id);
            }
        }

        // Batch fetch in chunks of 10
        let all_igdb_ids: Vec<i64> = igdb_id_to_rom_ids.keys().copied().collect();
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

        let _ = rom_ids; // suppress unused warning
    }

    // 2. Process each ROM
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

        // Step 1: Compute hash if missing
        let md5 = compute_md5_if_needed(pool, rom).await;

        // Step 2: Hasheous lookup (check cache first, then API)
        let hasheous_result = hasheous::get_cached(pool, rom.id).await;
        let hasheous_result = match hasheous_result {
            Some(cached) => Some(cached),
            None => {
                if let Some(ref hash) = md5 {
                    if let Some(result) = hasheous::lookup_by_md5(&http_client, hash).await {
                        hasheous::save_to_cache(pool, rom.id, &result).await;
                        Some(result)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };

        let hasheous_name = hasheous_result.as_ref().map(|r| r.name.as_str());

        // Upsert metadata from Hasheous if we got a result
        if let Some(ref result) = hasheous_result {
            let genres_json =
                serde_json::to_string(&result.genres).unwrap_or_else(|_| "[]".to_string());
            if let Err(e) = sqlx::query(
                "INSERT INTO metadata (rom_id, description, publisher, genres, release_date, metadata_fetched_at)
                 VALUES (?, ?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                 ON CONFLICT(rom_id) DO UPDATE SET
                   description = COALESCE(excluded.description, metadata.description),
                   publisher = COALESCE(excluded.publisher, metadata.publisher),
                   genres = CASE WHEN excluded.genres != '[]' THEN excluded.genres ELSE metadata.genres END,
                   release_date = COALESCE(excluded.release_date, metadata.release_date),
                   metadata_fetched_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
            )
            .bind(rom.id)
            .bind(&result.description)
            .bind(&result.publisher)
            .bind(&genres_json)
            .bind(&result.year)
            .execute(pool)
            .await
            {
                log::warn!("Failed to upsert Hasheous metadata for rom {}: {e}", rom.id);
            }
        }

        // Step 3: IGDB enrichment (if client provided)
        if igdb_client.is_some() {
            // Try to find IGDB data: first from batch, then individual search
            let igdb_game_id = sqlx::query_scalar::<_, Option<i64>>(
                "SELECT igdb_game_id FROM hasheous_cache WHERE rom_id = ? AND igdb_game_id IS NOT NULL",
            )
            .bind(rom.id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .flatten();

            let igdb_data = if let Some(igdb_id) = igdb_game_id {
                igdb_batch.get(&igdb_id).cloned()
            } else {
                None
            };

            // Fallback: search by name if no batch data and we have a client
            let igdb_data = if igdb_data.is_none() {
                if let Some(client) = igdb_client {
                    let search_name = hasheous_name.unwrap_or(&rom.name);
                    match client.search_game(search_name).await {
                        Ok(result) => result,
                        Err(e) => {
                            log::warn!("IGDB search failed for rom {}: {e}", rom.id);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                igdb_data
            };

            if let Some(ref game) = igdb_data {
                apply_igdb_data(pool, rom.id, game).await;
            }
        }

        // Step 4: LaunchBox lookup using verified name (or fall back to ROM name)
        let lb_game = if has_launchbox {
            let lookup_name = hasheous_name.unwrap_or(&rom.name);
            launchbox::find_by_name(pool, lookup_name, &rom.platform_slug).await
        } else {
            None
        };

        if let Some(ref lb_game) = lb_game {
            if let Err(e) = sqlx::query(
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
            )
            .bind(rom.id)
            .bind(&lb_game.overview)
            .bind(&lb_game.developer)
            .bind(&lb_game.publisher)
            .bind(&lb_game.genres)
            .bind(&lb_game.release_date)
            .bind(lb_game.community_rating)
            .execute(pool)
            .await
            {
                log::warn!("Failed to upsert LaunchBox metadata for rom {}: {e}", rom.id);
            }

            // Try LaunchBox cover if we don't have one yet
            if rom.has_cover == 0 {
                if let Some(url) = launchbox::get_image_url(pool, &lb_game.database_id).await {
                    if let Err(e) = sqlx::query(
                        "INSERT INTO artwork (rom_id, art_type, url)
                         VALUES (?, 'cover', ?)
                         ON CONFLICT(rom_id, art_type, url) DO NOTHING",
                    )
                    .bind(rom.id)
                    .bind(&url)
                    .execute(pool)
                    .await
                    {
                        log::warn!("Failed to insert LaunchBox artwork for rom {}: {e}", rom.id);
                    }
                }
            }
        }

        // Step 5: ScreenScraper enrichment (after LaunchBox, before libretro)
        if let Some(ss_system_id) = rom.screenscraper_id {
            if !screenscraper::is_cached(pool, rom.id).await {
                match screenscraper::lookup_game(
                    &http_client,
                    ss_creds,
                    md5.as_deref(),
                    &rom.name,
                    ss_system_id,
                )
                .await
                {
                    Ok(Some(ss_data)) => {
                        // Save raw response to cache
                        screenscraper::save_to_cache(
                            pool,
                            rom.id,
                            ss_data.game_id,
                            &serde_json::to_string(&ss_data.game_id).unwrap_or_default(),
                        )
                        .await;

                        // Apply metadata (only fill NULLs — COALESCE pattern)
                        apply_screenscraper_metadata(pool, rom.id, &ss_data).await;

                        // Always append artwork with ON CONFLICT DO NOTHING
                        apply_screenscraper_artwork(pool, rom.id, &ss_data.media).await;
                    }
                    Ok(None) => {
                        // Cache the miss so we don't re-query
                        screenscraper::save_to_cache(pool, rom.id, None, "").await;
                    }
                    Err(e) => {
                        log::warn!("ScreenScraper lookup failed for rom {}: {e}", rom.id);
                    }
                }
            }
        }

        // Step 6: libretro thumbnail (if still no cover)
        // Re-check cover status since LaunchBox/ScreenScraper might have set one
        let current_has_cover = if rom.has_cover == 0 {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM artwork WHERE rom_id = ? AND art_type = 'cover'",
            )
            .bind(rom.id)
            .fetch_one(pool)
            .await
            .unwrap_or(0)
                > 0
        } else {
            true
        };

        if !current_has_cover {
            let name = hasheous_name.unwrap_or(&rom.name);
            if let Some(url) = libretro_thumbnails::build_thumbnail_url(&rom.platform_slug, name) {
                let exists = http_client
                    .head(&url)
                    .send()
                    .await
                    .is_ok_and(|r| r.status().is_success());

                if exists {
                    if let Err(e) = sqlx::query(
                        "INSERT INTO artwork (rom_id, art_type, url)
                         VALUES (?, 'cover', ?)
                         ON CONFLICT(rom_id, art_type, url) DO NOTHING",
                    )
                    .bind(rom.id)
                    .bind(&url)
                    .execute(pool)
                    .await
                    {
                        log::warn!("Failed to insert libretro artwork for rom {}: {e}", rom.id);
                    }
                }
            }
        }

        // Step 6: Screenshot art — collect from all sources (libretro + LaunchBox)
        {
            let snap_name = hasheous_name.unwrap_or(&rom.name);

            // libretro Named_Snaps
            if let Some(url) =
                libretro_thumbnails::build_snapshot_url(&rom.platform_slug, snap_name)
            {
                let exists = http_client
                    .head(&url)
                    .send()
                    .await
                    .is_ok_and(|r| r.status().is_success());
                if exists {
                    if let Err(e) = sqlx::query(
                        "INSERT INTO artwork (rom_id, art_type, url)
                         VALUES (?, 'screenshot', ?)
                         ON CONFLICT(rom_id, art_type, url) DO NOTHING",
                    )
                    .bind(rom.id)
                    .bind(&url)
                    .execute(pool)
                    .await
                    {
                        log::warn!("Failed to insert screenshot for rom {}: {e}", rom.id);
                    }
                }
            }

            // libretro Named_Titles
            if let Some(url) =
                libretro_thumbnails::build_title_url(&rom.platform_slug, snap_name)
            {
                let exists = http_client
                    .head(&url)
                    .send()
                    .await
                    .is_ok_and(|r| r.status().is_success());
                if exists {
                    if let Err(e) = sqlx::query(
                        "INSERT INTO artwork (rom_id, art_type, url)
                         VALUES (?, 'screenshot', ?)
                         ON CONFLICT(rom_id, art_type, url) DO NOTHING",
                    )
                    .bind(rom.id)
                    .bind(&url)
                    .execute(pool)
                    .await
                    {
                        log::warn!("Failed to insert screenshot for rom {}: {e}", rom.id);
                    }
                }
            }

            // LaunchBox screenshots (multiple) — reuse lb_game from Step 3
            if let Some(ref lb_game) = lb_game {
                let urls =
                    launchbox::get_screenshot_urls(pool, &lb_game.database_id).await;
                for url in urls {
                    if let Err(e) = sqlx::query(
                        "INSERT INTO artwork (rom_id, art_type, url)
                         VALUES (?, 'screenshot', ?)
                         ON CONFLICT(rom_id, art_type, url) DO NOTHING",
                    )
                    .bind(rom.id)
                    .bind(&url)
                    .execute(pool)
                    .await
                    {
                        log::warn!("Failed to insert screenshot for rom {}: {e}", rom.id);
                    }
                }
            }
        }

        // Mark as enriched if not already
        if let Err(e) = sqlx::query(
            "INSERT INTO metadata (rom_id, metadata_fetched_at)
             VALUES (?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(rom_id) DO UPDATE SET
               metadata_fetched_at = COALESCE(metadata.metadata_fetched_at, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        )
        .bind(rom.id)
        .execute(pool)
        .await
        {
            log::warn!("Failed to mark rom {} as enriched: {e}", rom.id);
        }
    }

    Ok(())
}

/// Enrich a single ROM by ID — runs Hasheous lookup, IGDB, LaunchBox, ScreenScraper, and cover art.
/// Clears the existing hasheous_cache entry first so fresh data is fetched.
pub async fn enrich_single_rom(
    rom_id: i64,
    pool: &SqlitePool,
    igdb_client: Option<&igdb::IgdbClient>,
    ss_creds: Option<&screenscraper::SsUserCredentials>,
) -> AppResult<()> {
    let rom = sqlx::query_as::<_, RomRow>(
        "SELECT r.id, r.name, p.slug as platform_slug,
                (SELECT COUNT(*) FROM artwork WHERE rom_id = r.id AND art_type = 'cover') as has_cover,
                r.hash_md5,
                (SELECT s2.source_type FROM source_roms sr2 JOIN sources s2 ON s2.id = sr2.source_id WHERE sr2.rom_id = r.id LIMIT 1) as source_type,
                (SELECT sr3.source_rom_id FROM source_roms sr3 JOIN sources s3 ON s3.id = sr3.source_id WHERE sr3.rom_id = r.id LIMIT 1) as source_rom_id,
                p.screenscraper_id
         FROM roms r
         JOIN platforms p ON p.id = r.platform_id
         WHERE r.id = ?",
    )
    .bind(rom_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Other(format!("ROM {rom_id} not found")))?;

    // Clear existing hasheous cache so we re-fetch
    let _ = sqlx::query("DELETE FROM hasheous_cache WHERE rom_id = ?")
        .bind(rom_id)
        .execute(pool)
        .await;

    let http_client = reqwest::Client::builder()
        .user_agent("romm-buddy/0.1")
        .build()
        .unwrap_or_default();

    let has_launchbox = launchbox::has_imported_db(pool).await;

    // Step 1: Compute hash if missing
    let md5 = compute_md5_if_needed(pool, &rom).await;

    // Step 2: Hasheous lookup
    let hasheous_result = if let Some(ref hash) = md5 {
        if let Some(result) = hasheous::lookup_by_md5(&http_client, hash).await {
            hasheous::save_to_cache(pool, rom.id, &result).await;
            Some(result)
        } else {
            None
        }
    } else {
        None
    };

    let hasheous_name = hasheous_result.as_ref().map(|r| r.name.as_str());

    // Upsert metadata from Hasheous
    if let Some(ref result) = hasheous_result {
        let genres_json =
            serde_json::to_string(&result.genres).unwrap_or_else(|_| "[]".to_string());
        let _ = sqlx::query(
            "INSERT INTO metadata (rom_id, description, publisher, genres, release_date, metadata_fetched_at)
             VALUES (?, ?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(rom_id) DO UPDATE SET
               description = COALESCE(excluded.description, metadata.description),
               publisher = COALESCE(excluded.publisher, metadata.publisher),
               genres = CASE WHEN excluded.genres != '[]' THEN excluded.genres ELSE metadata.genres END,
               release_date = COALESCE(excluded.release_date, metadata.release_date),
               metadata_fetched_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
        )
        .bind(rom.id)
        .bind(&result.description)
        .bind(&result.publisher)
        .bind(&genres_json)
        .bind(&result.year)
        .execute(pool)
        .await;
    }

    // Step 3: IGDB enrichment (if client provided)
    if let Some(client) = igdb_client {
        let igdb_game_id = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT igdb_game_id FROM hasheous_cache WHERE rom_id = ? AND igdb_game_id IS NOT NULL",
        )
        .bind(rom.id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .flatten();

        let igdb_data = if let Some(igdb_id) = igdb_game_id {
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
        };

        if let Some(ref game) = igdb_data {
            apply_igdb_data(pool, rom.id, game).await;
        }
    }

    // Step 4: LaunchBox lookup
    let lb_game = if has_launchbox {
        let lookup_name = hasheous_name.unwrap_or(&rom.name);
        launchbox::find_by_name(pool, lookup_name, &rom.platform_slug).await
    } else {
        None
    };

    if let Some(ref lb_game) = lb_game {
        let _ = sqlx::query(
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
        )
        .bind(rom.id)
        .bind(&lb_game.overview)
        .bind(&lb_game.developer)
        .bind(&lb_game.publisher)
        .bind(&lb_game.genres)
        .bind(&lb_game.release_date)
        .bind(lb_game.community_rating)
        .execute(pool)
        .await;

        if rom.has_cover == 0 {
            if let Some(url) = launchbox::get_image_url(pool, &lb_game.database_id).await {
                let _ = sqlx::query(
                    "INSERT INTO artwork (rom_id, art_type, url)
                     VALUES (?, 'cover', ?)
                     ON CONFLICT(rom_id, art_type, url) DO NOTHING",
                )
                .bind(rom.id)
                .bind(&url)
                .execute(pool)
                .await;
            }
        }
    }

    // Step 5: ScreenScraper enrichment (after LaunchBox, before libretro)
    if let Some(ss_system_id) = rom.screenscraper_id {
        // Clear screenscraper cache on re-enrich (same as hasheous)
        let _ = sqlx::query("DELETE FROM screenscraper_cache WHERE rom_id = ?")
            .bind(rom.id)
            .execute(pool)
            .await;

        match screenscraper::lookup_game(
            &http_client,
            ss_creds,
            md5.as_deref(),
            &rom.name,
            ss_system_id,
        )
        .await
        {
            Ok(Some(ss_data)) => {
                screenscraper::save_to_cache(
                    pool,
                    rom.id,
                    ss_data.game_id,
                    &serde_json::to_string(&ss_data.game_id).unwrap_or_default(),
                )
                .await;
                apply_screenscraper_metadata(pool, rom.id, &ss_data).await;
                apply_screenscraper_artwork(pool, rom.id, &ss_data.media).await;
            }
            Ok(None) => {
                screenscraper::save_to_cache(pool, rom.id, None, "").await;
            }
            Err(e) => {
                log::warn!("ScreenScraper lookup failed for rom {}: {e}", rom.id);
            }
        }
    }

    // Step 6: libretro thumbnail if no cover
    let current_has_cover = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM artwork WHERE rom_id = ? AND art_type = 'cover'",
    )
    .bind(rom.id)
    .fetch_one(pool)
    .await
    .unwrap_or(0)
        > 0;

    if !current_has_cover {
        let name = hasheous_name.unwrap_or(&rom.name);
        if let Some(url) = libretro_thumbnails::build_thumbnail_url(&rom.platform_slug, name) {
            let exists = http_client
                .head(&url)
                .send()
                .await
                .is_ok_and(|r| r.status().is_success());
            if exists {
                let _ = sqlx::query(
                    "INSERT INTO artwork (rom_id, art_type, url)
                     VALUES (?, 'cover', ?)
                     ON CONFLICT(rom_id, art_type, url) DO NOTHING",
                )
                .bind(rom.id)
                .bind(&url)
                .execute(pool)
                .await;
            }
        }
    }

    // Step 5: Screenshot art — collect from all sources (libretro + LaunchBox)
    // Clear existing screenshots on re-enrich so we get fresh data
    let _ = sqlx::query("DELETE FROM artwork WHERE rom_id = ? AND art_type = 'screenshot'")
        .bind(rom.id)
        .execute(pool)
        .await;

    let snap_name = hasheous_name.unwrap_or(&rom.name);

    // libretro Named_Snaps
    if let Some(url) = libretro_thumbnails::build_snapshot_url(&rom.platform_slug, snap_name) {
        let exists = http_client
            .head(&url)
            .send()
            .await
            .is_ok_and(|r| r.status().is_success());
        if exists {
            if let Err(e) = sqlx::query(
                "INSERT INTO artwork (rom_id, art_type, url)
                 VALUES (?, 'screenshot', ?)
                 ON CONFLICT(rom_id, art_type, url) DO NOTHING",
            )
            .bind(rom.id)
            .bind(&url)
            .execute(pool)
            .await
            {
                log::warn!("Failed to insert screenshot for rom {}: {e}", rom.id);
            }
        }
    }

    // libretro Named_Titles
    if let Some(url) = libretro_thumbnails::build_title_url(&rom.platform_slug, snap_name) {
        let exists = http_client
            .head(&url)
            .send()
            .await
            .is_ok_and(|r| r.status().is_success());
        if exists {
            if let Err(e) = sqlx::query(
                "INSERT INTO artwork (rom_id, art_type, url)
                 VALUES (?, 'screenshot', ?)
                 ON CONFLICT(rom_id, art_type, url) DO NOTHING",
            )
            .bind(rom.id)
            .bind(&url)
            .execute(pool)
            .await
            {
                log::warn!("Failed to insert screenshot for rom {}: {e}", rom.id);
            }
        }
    }

    // LaunchBox screenshots (multiple) — reuse lb_game from Step 3
    if let Some(ref lb_game) = lb_game {
        let urls = launchbox::get_screenshot_urls(pool, &lb_game.database_id).await;
        for url in urls {
            if let Err(e) = sqlx::query(
                "INSERT INTO artwork (rom_id, art_type, url)
                 VALUES (?, 'screenshot', ?)
                 ON CONFLICT(rom_id, art_type, url) DO NOTHING",
            )
            .bind(rom.id)
            .bind(&url)
            .execute(pool)
            .await
            {
                log::warn!("Failed to insert screenshot for rom {}: {e}", rom.id);
            }
        }
    }

    Ok(())
}

/// Apply IGDB game data to database: insert into igdb_cache, update metadata, save artwork.
async fn apply_igdb_data(pool: &SqlitePool, rom_id: i64, game: &igdb::IgdbGameData) {
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

    if let Err(e) = sqlx::query(
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
    )
    .bind(rom_id)
    .bind(game.id)
    .bind(&game.name)
    .bind(&game.summary)
    .bind(&game.storyline)
    .bind(game.aggregated_rating)
    .bind(game.first_release_date_string())
    .bind(&genres_json)
    .bind(&themes_json)
    .bind(&game_modes_json)
    .bind(&player_perspectives_json)
    .bind(game.developer())
    .bind(game.publisher())
    .bind(game.cover_image_id())
    .bind(&screenshot_ids_json)
    .bind(game.franchise_name())
    .bind(&raw_response)
    .execute(pool)
    .await
    {
        log::warn!("Failed to insert IGDB cache for rom {rom_id}: {e}");
    }

    // Update metadata table — IGDB overrides description, rating, genres, themes, developer, publisher
    let description = game.description();
    let rating = game.aggregated_rating.map(|r| r / 10.0); // IGDB is 0-100, normalize to 0-10
    let release_date = game.first_release_date_string();

    if let Err(e) = sqlx::query(
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
    )
    .bind(rom_id)
    .bind(&description)
    .bind(game.developer())
    .bind(game.publisher())
    .bind(&genres_json)
    .bind(&themes_json)
    .bind(rating)
    .bind(&release_date)
    .bind(game.id)
    .execute(pool)
    .await
    {
        log::warn!("Failed to upsert IGDB metadata for rom {rom_id}: {e}");
    }

    // Save IGDB cover art
    if let Some(url) = game.cover_url() {
        if let Err(e) = sqlx::query(
            "INSERT INTO artwork (rom_id, art_type, url)
             VALUES (?, 'cover', ?)
             ON CONFLICT(rom_id, art_type, url) DO NOTHING",
        )
        .bind(rom_id)
        .bind(&url)
        .execute(pool)
        .await
        {
            log::warn!("Failed to insert IGDB cover for rom {rom_id}: {e}");
        }
    }

    // Save IGDB screenshots
    for url in game.screenshot_urls() {
        if let Err(e) = sqlx::query(
            "INSERT INTO artwork (rom_id, art_type, url)
             VALUES (?, 'screenshot', ?)
             ON CONFLICT(rom_id, art_type, url) DO NOTHING",
        )
        .bind(rom_id)
        .bind(&url)
        .execute(pool)
        .await
        {
            log::warn!("Failed to insert IGDB screenshot for rom {rom_id}: {e}");
        }
    }
}

/// Apply ScreenScraper metadata to database (only fill NULLs).
async fn apply_screenscraper_metadata(
    pool: &SqlitePool,
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

    if let Err(e) = sqlx::query(
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
    )
    .bind(rom_id)
    .bind(&data.synopsis)
    .bind(&data.developer)
    .bind(&data.publisher)
    .bind(&genres_json)
    .bind(&data.release_date)
    .bind(data.rating)
    .execute(pool)
    .await
    {
        log::warn!("Failed to upsert ScreenScraper metadata for rom {rom_id}: {e}");
    }
}

/// Apply ScreenScraper artwork (always append with ON CONFLICT DO NOTHING).
async fn apply_screenscraper_artwork(
    pool: &SqlitePool,
    rom_id: i64,
    media: &[screenscraper::SsMedia],
) {
    for item in media {
        if let Err(e) = sqlx::query(
            "INSERT INTO artwork (rom_id, art_type, url)
             VALUES (?, ?, ?)
             ON CONFLICT(rom_id, art_type, url) DO NOTHING",
        )
        .bind(rom_id)
        .bind(&item.media_type)
        .bind(&item.url)
        .execute(pool)
        .await
        {
            log::warn!(
                "Failed to insert ScreenScraper artwork for rom {rom_id}: {e}"
            );
        }
    }
}
