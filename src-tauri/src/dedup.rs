use sqlx::SqlitePool;

use crate::error::AppResult;

/// Check if a ROM with this hash already exists on this platform.
pub async fn find_existing_rom_by_hash(
    pool: &SqlitePool,
    platform_id: i64,
    hash_md5: &str,
) -> AppResult<Option<i64>> {
    let rom_id = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM roms WHERE platform_id = ? AND hash_md5 = ? LIMIT 1",
    )
    .bind(platform_id)
    .bind(hash_md5)
    .fetch_optional(pool)
    .await?;
    Ok(rom_id)
}

/// Check if a ROM with this filename already exists on this platform.
pub async fn find_existing_rom_by_filename(
    pool: &SqlitePool,
    platform_id: i64,
    file_name: &str,
) -> AppResult<Option<i64>> {
    let rom_id = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM roms WHERE platform_id = ? AND file_name = ? LIMIT 1",
    )
    .bind(platform_id)
    .bind(file_name)
    .fetch_optional(pool)
    .await?;
    Ok(rom_id)
}

/// Insert or link a ROM with deduplication.
///
/// Priority:
/// 1. Hash match → add source_roms link to existing ROM
/// 2. Filename match → upsert existing ROM, add source_roms link
/// 3. No match → insert new ROM + source_roms link
///
/// Returns the ROM id.
#[allow(clippy::too_many_arguments)]
pub async fn upsert_rom_deduped(
    pool: &SqlitePool,
    platform_id: i64,
    name: &str,
    file_name: &str,
    file_size: Option<i64>,
    regions: &str,
    hash_md5: Option<&str>,
    source_id: i64,
    source_rom_id: Option<&str>,
    source_url: Option<&str>,
) -> AppResult<i64> {
    // Phase 1: Check by hash (if available)
    if let Some(hash) = hash_md5 {
        if !hash.is_empty() {
            if let Some(rom_id) = find_existing_rom_by_hash(pool, platform_id, hash).await? {
                // ROM exists by hash — just link the source
                link_source(pool, rom_id, source_id, source_rom_id, source_url, Some(file_name), hash_md5).await?;
                return Ok(rom_id);
            }
        }
    }

    // Phase 2: Check by filename
    if let Some(rom_id) = find_existing_rom_by_filename(pool, platform_id, file_name).await? {
        // Upsert: update metadata if richer
        sqlx::query(
            "UPDATE roms SET
                name = COALESCE(NULLIF(?, ''), name),
                file_size = COALESCE(?, file_size),
                regions = CASE WHEN ? != '[]' THEN ? ELSE regions END,
                hash_md5 = COALESCE(?, hash_md5),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
        )
        .bind(name)
        .bind(file_size)
        .bind(regions)
        .bind(regions)
        .bind(hash_md5)
        .bind(rom_id)
        .execute(pool)
        .await?;

        link_source(pool, rom_id, source_id, source_rom_id, source_url, Some(file_name), hash_md5).await?;
        return Ok(rom_id);
    }

    // Phase 3: New ROM
    let rom_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO roms (platform_id, name, file_name, file_size, regions, hash_md5)
         VALUES (?, ?, ?, ?, ?, ?)
         RETURNING id",
    )
    .bind(platform_id)
    .bind(name)
    .bind(file_name)
    .bind(file_size)
    .bind(regions)
    .bind(hash_md5)
    .fetch_one(pool)
    .await?;

    link_source(pool, rom_id, source_id, source_rom_id, source_url, Some(file_name), hash_md5).await?;
    Ok(rom_id)
}

/// Create or update a source_roms link.
async fn link_source(
    pool: &SqlitePool,
    rom_id: i64,
    source_id: i64,
    source_rom_id: Option<&str>,
    source_url: Option<&str>,
    file_name: Option<&str>,
    hash_md5: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO source_roms (rom_id, source_id, source_rom_id, source_url, file_name, hash_md5)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(rom_id, source_id) DO UPDATE SET
           source_rom_id = COALESCE(excluded.source_rom_id, source_roms.source_rom_id),
           source_url = COALESCE(excluded.source_url, source_roms.source_url),
           file_name = COALESCE(excluded.file_name, source_roms.file_name),
           hash_md5 = COALESCE(excluded.hash_md5, source_roms.hash_md5)",
    )
    .bind(rom_id)
    .bind(source_id)
    .bind(source_rom_id)
    .bind(source_url)
    .bind(file_name)
    .bind(hash_md5)
    .execute(pool)
    .await?;
    Ok(())
}

/// Post-enrichment reconciliation: find ROMs sharing (platform_id, hash_md5)
/// and merge them (keep oldest, move all related rows, delete dupes).
pub async fn reconcile_duplicates(pool: &SqlitePool) -> AppResult<u64> {
    // Find duplicate groups
    let groups = sqlx::query_as::<_, (i64, String)>(
        "SELECT platform_id, hash_md5
         FROM roms
         WHERE hash_md5 IS NOT NULL AND hash_md5 != ''
         GROUP BY platform_id, hash_md5
         HAVING COUNT(*) > 1",
    )
    .fetch_all(pool)
    .await?;

    let mut merged_count: u64 = 0;

    for (platform_id, hash_md5) in &groups {
        // Get all ROM IDs in this group, ordered by id (keep oldest)
        let rom_ids = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM roms WHERE platform_id = ? AND hash_md5 = ? ORDER BY id",
        )
        .bind(platform_id)
        .bind(hash_md5)
        .fetch_all(pool)
        .await?;

        if rom_ids.len() < 2 {
            continue;
        }

        let keeper_id = rom_ids[0];
        let dupes = &rom_ids[1..];

        for &dupe_id in dupes {
            // Move source_roms links to keeper (ignore conflicts — keeper may already have that source)
            sqlx::query(
                "UPDATE OR IGNORE source_roms SET rom_id = ? WHERE rom_id = ?",
            )
            .bind(keeper_id)
            .bind(dupe_id)
            .execute(pool)
            .await?;

            // Move metadata (if keeper doesn't have it)
            sqlx::query(
                "UPDATE OR IGNORE metadata SET rom_id = ? WHERE rom_id = ?",
            )
            .bind(keeper_id)
            .bind(dupe_id)
            .execute(pool)
            .await?;

            // Move artwork (ignore conflicts)
            sqlx::query(
                "UPDATE OR IGNORE artwork SET rom_id = ? WHERE rom_id = ?",
            )
            .bind(keeper_id)
            .bind(dupe_id)
            .execute(pool)
            .await?;

            // Move library entries (ignore conflicts)
            sqlx::query(
                "UPDATE OR IGNORE library SET rom_id = ? WHERE rom_id = ?",
            )
            .bind(keeper_id)
            .bind(dupe_id)
            .execute(pool)
            .await?;

            // Move hasheous_cache (ignore conflicts)
            sqlx::query(
                "UPDATE OR IGNORE hasheous_cache SET rom_id = ? WHERE rom_id = ?",
            )
            .bind(keeper_id)
            .bind(dupe_id)
            .execute(pool)
            .await?;

            // Delete the duplicate ROM (CASCADE will clean up orphaned rows)
            sqlx::query("DELETE FROM roms WHERE id = ?")
                .bind(dupe_id)
                .execute(pool)
                .await?;

            merged_count += 1;
        }
    }

    Ok(merged_count)
}
