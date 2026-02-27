use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

use crate::error::{AppError, AppResult};
use crate::hash;
use crate::models::ScanProgress;

/// Maps No-Intro / Redump DAT header names to canonical platform slugs.
static DAT_NAME_TO_SLUG: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([
        // No-Intro names
        ("Nintendo - Game Boy", "gb"),
        ("Nintendo - Game Boy Color", "gbc"),
        ("Nintendo - Game Boy Advance", "gba"),
        ("Nintendo - Nintendo Entertainment System", "nes"),
        ("Nintendo - Super Nintendo Entertainment System", "snes"),
        ("Nintendo - Nintendo 64", "n64"),
        ("Nintendo - Nintendo DS", "nds"),
        ("Nintendo - Virtual Boy", "vb"),
        ("Sega - Mega Drive - Genesis", "genesis"),
        ("Sega - Game Gear", "gamegear"),
        ("Sega - Master System - Mark III", "mastersystem"),
        ("Sega - SG-1000", "sg1000"),
        ("NEC - PC Engine - TurboGrafx-16", "pce"),
        ("NEC - PC Engine SuperGrafx", "sgfx"),
        ("Atari - 2600", "atari2600"),
        ("Atari - 5200", "atari5200"),
        ("Atari - 7800", "atari7800"),
        ("Atari - Lynx", "lynx"),
        ("SNK - Neo Geo Pocket", "ngp"),
        ("SNK - Neo Geo Pocket Color", "ngpc"),
        ("SNK - Neo Geo CD", "neocd"),
        ("SNK - Neo Geo", "neogeo"),
        ("Bandai - WonderSwan", "ws"),
        ("Bandai - WonderSwan Color", "wsc"),
        ("Coleco - ColecoVision", "colecovision"),
        ("GCE - Vectrex", "vectrex"),
        ("Mattel - Intellivision", "intellivision"),
        ("Nintendo - Pokemon Mini", "pokemini"),
        ("Nintendo - Famicom Disk System", "fds"),
        // Redump names
        ("Sony - PlayStation", "psx"),
        ("Sony - PlayStation 2", "ps2"),
        ("Sony - PlayStation Portable", "psp"),
        ("Sega - Dreamcast", "dreamcast"),
        ("Sega - Saturn", "saturn"),
        ("Sega - Mega-CD - Sega CD", "segacd"),
        ("NEC - PC Engine CD - TurboGrafx-CD", "pcecd"),
        ("NEC - PC-FX", "pcfx"),
        ("Nintendo - GameCube", "gamecube"),
        ("Nintendo - Wii", "wii"),
        ("Panasonic - 3DO Interactive Multiplayer", "3do"),
        ("Philips - CD-i", "cdi"),
    ])
});

/// Parsed DAT header info.
pub struct DatHeader {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
}

/// A single ROM entry from a DAT file.
pub struct DatEntry {
    pub game_name: String,
    pub rom_name: String,
    pub size: Option<i64>,
    pub crc32: Option<String>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub status: Option<String>,
}

/// Result of parsing a DAT file.
pub struct ParsedDat {
    pub header: DatHeader,
    pub entries: Vec<DatEntry>,
}

/// Info about an imported DAT file (returned to frontend).
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct DatFileInfo {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub dat_type: String,
    pub platform_slug: String,
    pub entry_count: i64,
    pub imported_at: String,
}

/// Verification summary stats.
#[derive(Debug, serde::Serialize)]
pub struct VerificationStats {
    pub verified: i64,
    pub unverified: i64,
    pub bad_dump: i64,
    pub not_checked: i64,
}

/// Auto-detect platform slug from DAT header name.
pub fn detect_platform_slug(dat_name: &str) -> Option<String> {
    DAT_NAME_TO_SLUG
        .get(dat_name)
        .map(|s| (*s).to_string())
}

/// Parse a Logiqx XML DAT file, returning header + entries.
pub fn parse_dat_file(path: &Path) -> AppResult<ParsedDat> {
    let file = std::fs::File::open(path)?;
    let reader_buf = std::io::BufReader::with_capacity(64 * 1024, file);
    let mut reader = Reader::from_reader(reader_buf);
    reader.config_mut().trim_text(true);

    let mut header = DatHeader {
        name: String::new(),
        description: None,
        version: None,
    };
    let mut entries: Vec<DatEntry> = Vec::new();
    let mut buf = Vec::with_capacity(4096);

    #[derive(PartialEq)]
    enum Section { None, Header, Game }

    let mut section = Section::None;
    let mut current_element = String::new();
    let mut current_game_name = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "header" => section = Section::Header,
                    "game" | "machine" => {
                        section = Section::Game;
                        current_game_name.clear();
                        // Get game name from attribute
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"name" {
                                current_game_name = attr.unescape_value()
                                    .unwrap_or_default()
                                    .to_string();
                            }
                        }
                    }
                    _ => {}
                }
                current_element = tag;
            }
            Ok(Event::Empty(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "rom" && section == Section::Game {
                    let mut entry = DatEntry {
                        game_name: current_game_name.clone(),
                        rom_name: String::new(),
                        size: None,
                        crc32: None,
                        md5: None,
                        sha1: None,
                        status: None,
                    };
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"name" => {
                                entry.rom_name = attr.unescape_value()
                                    .unwrap_or_default()
                                    .to_string();
                            }
                            b"size" => {
                                entry.size = attr.unescape_value()
                                    .ok()
                                    .and_then(|v| v.parse().ok());
                            }
                            b"crc" => {
                                entry.crc32 = Some(
                                    attr.unescape_value()
                                        .unwrap_or_default()
                                        .to_uppercase(),
                                );
                            }
                            b"md5" => {
                                entry.md5 = Some(
                                    attr.unescape_value()
                                        .unwrap_or_default()
                                        .to_lowercase(),
                                );
                            }
                            b"sha1" => {
                                entry.sha1 = Some(
                                    attr.unescape_value()
                                        .unwrap_or_default()
                                        .to_lowercase(),
                                );
                            }
                            b"status" => {
                                entry.status = Some(
                                    attr.unescape_value()
                                        .unwrap_or_default()
                                        .to_string(),
                                );
                            }
                            _ => {}
                        }
                    }
                    entries.push(entry);
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if section == Section::Header {
                    match current_element.as_str() {
                        "name" => header.name = text,
                        "description" => header.description = Some(text),
                        "version" => header.version = Some(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "header" => section = Section::None,
                    "game" | "machine" => section = Section::None,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(AppError::Other(format!("XML parse error: {e}")));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(ParsedDat { header, entries })
}

/// Import a DAT file into the database. Returns the dat_file id.
pub async fn import_dat_file(
    pool: &SqlitePool,
    path: &Path,
    dat_type: &str,
    platform_slug: &str,
    on_progress: impl Fn(ScanProgress) + Send + 'static,
) -> AppResult<i64> {
    let path = path.to_path_buf();
    let dat_type = dat_type.to_string();
    let platform_slug = platform_slug.to_string();

    // Parse in blocking task
    let parsed = tokio::task::spawn_blocking(move || parse_dat_file(&path))
        .await
        .map_err(|e| AppError::Other(format!("Task join error: {e}")))?
        ?;

    on_progress(ScanProgress {
        source_id: -1,
        total: 1,
        current: 0,
        current_item: format!("Importing {} entries...", parsed.entries.len()),
    });

    // Remove any existing DAT for this platform + type
    sqlx::query(
        "DELETE FROM dat_files WHERE platform_slug = ? AND dat_type = ?",
    )
    .bind(&platform_slug)
    .bind(&dat_type)
    .execute(pool)
    .await?;

    // Insert dat_file record
    #[allow(clippy::cast_possible_wrap)]
    let entry_count = parsed.entries.len() as i64;
    let dat_file_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO dat_files (name, description, version, dat_type, platform_slug, entry_count)
         VALUES (?, ?, ?, ?, ?, ?)
         RETURNING id",
    )
    .bind(&parsed.header.name)
    .bind(&parsed.header.description)
    .bind(&parsed.header.version)
    .bind(&dat_type)
    .bind(&platform_slug)
    .bind(entry_count)
    .fetch_one(pool)
    .await?;

    // Batch insert entries
    let batch_size = 500;
    for (i, chunk) in parsed.entries.chunks(batch_size).enumerate() {
        let mut query = String::from(
            "INSERT INTO dat_entries (dat_file_id, game_name, rom_name, size, crc32, md5, sha1, status) VALUES ",
        );
        let mut first = true;
        for _ in chunk {
            if !first { query.push(','); }
            query.push_str("(?, ?, ?, ?, ?, ?, ?, ?)");
            first = false;
        }

        let mut q = sqlx::query(&query);
        for entry in chunk {
            q = q.bind(dat_file_id)
                .bind(&entry.game_name)
                .bind(&entry.rom_name)
                .bind(entry.size)
                .bind(&entry.crc32)
                .bind(&entry.md5)
                .bind(&entry.sha1)
                .bind(&entry.status);
        }
        q.execute(pool).await?;

        #[allow(clippy::cast_possible_truncation)]
        let progress_current = ((i + 1) * batch_size).min(parsed.entries.len()) as u64;
        on_progress(ScanProgress {
            source_id: -1,
            #[allow(clippy::cast_possible_truncation)]
            total: parsed.entries.len() as u64,
            current: progress_current,
            current_item: format!("Imported {progress_current} / {} entries", parsed.entries.len()),
        });
    }

    Ok(dat_file_id)
}

/// Verify ROMs against imported DAT files.
/// Computes triple hashes for ROMs, looks up in dat_entries, sets verification_status.
pub async fn verify_roms(
    pool: &SqlitePool,
    platform_id: Option<i64>,
    on_progress: impl Fn(ScanProgress) + Send,
    cancel: CancellationToken,
) -> AppResult<VerificationStats> {
    // Get ROMs that need verification (local ROMs with file paths)
    let roms = if let Some(pid) = platform_id {
        sqlx::query_as::<_, (i64, String, Option<String>, Option<String>, Option<String>, Option<String>)>(
            "SELECT r.id, r.name, r.hash_crc32, r.hash_md5, r.hash_sha1, sr.source_rom_id
             FROM roms r
             LEFT JOIN source_roms sr ON sr.rom_id = r.id
             LEFT JOIN sources s ON s.id = sr.source_id AND s.source_type = 'local'
             WHERE r.platform_id = ?
             GROUP BY r.id",
        )
        .bind(pid)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, (i64, String, Option<String>, Option<String>, Option<String>, Option<String>)>(
            "SELECT r.id, r.name, r.hash_crc32, r.hash_md5, r.hash_sha1, sr.source_rom_id
             FROM roms r
             LEFT JOIN source_roms sr ON sr.rom_id = r.id
             LEFT JOIN sources s ON s.id = sr.source_id AND s.source_type = 'local'
             GROUP BY r.id",
        )
        .fetch_all(pool)
        .await?
    };

    #[allow(clippy::cast_possible_truncation)]
    let total = roms.len() as u64;
    let mut stats = VerificationStats {
        verified: 0,
        unverified: 0,
        bad_dump: 0,
        not_checked: 0,
    };

    for (i, (rom_id, rom_name, existing_crc, existing_md5, existing_sha1, source_rom_id)) in roms.iter().enumerate() {
        if cancel.is_cancelled() {
            return Ok(stats);
        }

        #[allow(clippy::cast_possible_truncation)]
        let current = i as u64 + 1;
        if i % 10 == 0 {
            on_progress(ScanProgress {
                source_id: -1,
                total,
                current,
                current_item: format!("Verifying: {rom_name}"),
            });
        }

        // Compute hashes if missing and file is accessible
        let (crc, md5, sha1) = if existing_crc.is_some() && existing_md5.is_some() && existing_sha1.is_some() {
            (existing_crc.clone(), existing_md5.clone(), existing_sha1.clone())
        } else if let Some(ref path_str) = source_rom_id {
            let path = std::path::PathBuf::from(path_str);
            if path.exists() {
                let path_clone = path.clone();
                let hashes = tokio::task::spawn_blocking(move || {
                    hash::compute_triple_hash(&path_clone)
                })
                .await
                .ok()
                .and_then(|r| r.ok());

                if let Some(h) = hashes {
                    // Store computed hashes
                    let _ = sqlx::query(
                        "UPDATE roms SET hash_crc32 = ?, hash_md5 = ?, hash_sha1 = ? WHERE id = ?",
                    )
                    .bind(&h.crc32)
                    .bind(&h.md5)
                    .bind(&h.sha1)
                    .bind(rom_id)
                    .execute(pool)
                    .await;

                    (Some(h.crc32), Some(h.md5), Some(h.sha1))
                } else {
                    stats.not_checked += 1;
                    continue;
                }
            } else {
                stats.not_checked += 1;
                continue;
            }
        } else {
            // No file accessible, try with whatever hashes we have
            if existing_md5.is_none() && existing_crc.is_none() && existing_sha1.is_none() {
                stats.not_checked += 1;
                continue;
            }
            (existing_crc.clone(), existing_md5.clone(), existing_sha1.clone())
        };

        // Look up in dat_entries by any available hash
        let dat_match = find_dat_match(pool, &crc, &md5, &sha1).await?;

        match dat_match {
            Some((entry_id, game_name, status)) => {
                let verification = if status.as_deref() == Some("baddump") {
                    stats.bad_dump += 1;
                    "bad_dump"
                } else {
                    stats.verified += 1;
                    "verified"
                };
                sqlx::query(
                    "UPDATE roms SET verification_status = ?, dat_entry_id = ?, dat_game_name = ? WHERE id = ?",
                )
                .bind(verification)
                .bind(entry_id)
                .bind(&game_name)
                .bind(rom_id)
                .execute(pool)
                .await?;
            }
            None => {
                // Hashes computed but no DAT match
                if crc.is_some() || md5.is_some() || sha1.is_some() {
                    sqlx::query(
                        "UPDATE roms SET verification_status = 'unverified' WHERE id = ?",
                    )
                    .bind(rom_id)
                    .execute(pool)
                    .await?;
                    stats.unverified += 1;
                } else {
                    stats.not_checked += 1;
                }
            }
        }
    }

    Ok(stats)
}

/// Find a matching DAT entry by hash (try SHA1 first, then MD5, then CRC32).
async fn find_dat_match(
    pool: &SqlitePool,
    crc: &Option<String>,
    md5: &Option<String>,
    sha1: &Option<String>,
) -> AppResult<Option<(i64, String, Option<String>)>> {
    // SHA1 is most reliable
    if let Some(ref sha1_val) = sha1 {
        if let Some(row) = sqlx::query_as::<_, (i64, String, Option<String>)>(
            "SELECT id, game_name, status FROM dat_entries WHERE sha1 = ? LIMIT 1",
        )
        .bind(sha1_val)
        .fetch_optional(pool)
        .await?
        {
            return Ok(Some(row));
        }
    }

    // MD5
    if let Some(ref md5_val) = md5 {
        if let Some(row) = sqlx::query_as::<_, (i64, String, Option<String>)>(
            "SELECT id, game_name, status FROM dat_entries WHERE md5 = ? LIMIT 1",
        )
        .bind(md5_val)
        .fetch_optional(pool)
        .await?
        {
            return Ok(Some(row));
        }
    }

    // CRC32
    if let Some(ref crc_val) = crc {
        if let Some(row) = sqlx::query_as::<_, (i64, String, Option<String>)>(
            "SELECT id, game_name, status FROM dat_entries WHERE crc32 = ? LIMIT 1",
        )
        .bind(crc_val)
        .fetch_optional(pool)
        .await?
        {
            return Ok(Some(row));
        }
    }

    Ok(None)
}

/// Get verification summary stats for a platform (or all).
pub async fn get_verification_stats(
    pool: &SqlitePool,
    platform_id: Option<i64>,
) -> AppResult<VerificationStats> {
    let (verified, unverified, bad_dump, not_checked) = if let Some(pid) = platform_id {
        let v = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM roms WHERE platform_id = ? AND verification_status = 'verified'",
        ).bind(pid).fetch_one(pool).await?;
        let u = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM roms WHERE platform_id = ? AND verification_status = 'unverified'",
        ).bind(pid).fetch_one(pool).await?;
        let b = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM roms WHERE platform_id = ? AND verification_status = 'bad_dump'",
        ).bind(pid).fetch_one(pool).await?;
        let n = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM roms WHERE platform_id = ? AND verification_status IS NULL",
        ).bind(pid).fetch_one(pool).await?;
        (v, u, b, n)
    } else {
        let v = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM roms WHERE verification_status = 'verified'",
        ).fetch_one(pool).await?;
        let u = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM roms WHERE verification_status = 'unverified'",
        ).fetch_one(pool).await?;
        let b = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM roms WHERE verification_status = 'bad_dump'",
        ).fetch_one(pool).await?;
        let n = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM roms WHERE verification_status IS NULL",
        ).fetch_one(pool).await?;
        (v, u, b, n)
    };

    Ok(VerificationStats {
        verified,
        unverified,
        bad_dump,
        not_checked,
    })
}
