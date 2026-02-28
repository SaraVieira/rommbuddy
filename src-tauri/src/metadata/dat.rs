use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseBackend,
    DatabaseConnection, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter, Statement,
};
use tokio_util::sync::CancellationToken;

use crate::entity::{dat_entries, dat_files, roms};
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
#[derive(Debug, serde::Serialize)]
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
    db: &DatabaseConnection,
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
    dat_files::Entity::delete_many()
        .filter(dat_files::Column::PlatformSlug.eq(&platform_slug))
        .filter(dat_files::Column::DatType.eq(&dat_type))
        .exec(db)
        .await?;

    // Insert dat_file record
    #[allow(clippy::cast_possible_wrap)]
    let entry_count = parsed.entries.len() as i64;
    let result = dat_files::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        name: Set(parsed.header.name.clone()),
        description: Set(parsed.header.description.clone()),
        version: Set(parsed.header.version.clone()),
        dat_type: Set(dat_type.clone()),
        platform_slug: Set(platform_slug.clone()),
        entry_count: Set(entry_count),
        imported_at: Set(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
    }
    .insert(db)
    .await?;
    let dat_file_id = result.id;

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

        let mut values: Vec<sea_orm::Value> = Vec::new();
        for entry in chunk {
            values.push(dat_file_id.into());
            values.push(entry.game_name.clone().into());
            values.push(entry.rom_name.clone().into());
            values.push(entry.size.into());
            values.push(entry.crc32.clone().into());
            values.push(entry.md5.clone().into());
            values.push(entry.sha1.clone().into());
            values.push(entry.status.clone().into());
        }
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            &query,
            values,
        ))
        .await?;

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

/// Row returned by the verification ROM query.
#[derive(Debug, FromQueryResult)]
struct VerifyRomRow {
    id: i64,
    name: String,
    hash_crc32: Option<String>,
    hash_md5: Option<String>,
    hash_sha1: Option<String>,
    source_rom_id: Option<String>,
}

/// Verify ROMs against imported DAT files.
/// Computes triple hashes for ROMs, looks up in dat_entries, sets verification_status.
pub async fn verify_roms(
    db: &DatabaseConnection,
    platform_id: Option<i64>,
    on_progress: impl Fn(ScanProgress) + Send,
    cancel: CancellationToken,
) -> AppResult<VerificationStats> {
    // Get ROMs that need verification (local ROMs with file paths)
    let query = if let Some(pid) = platform_id {
        Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT r.id, r.name, r.hash_crc32, r.hash_md5, r.hash_sha1, sr.source_rom_id
             FROM roms r
             LEFT JOIN source_roms sr ON sr.rom_id = r.id
             LEFT JOIN sources s ON s.id = sr.source_id AND s.source_type = 'local'
             WHERE r.platform_id = ?
             GROUP BY r.id",
            [pid.into()],
        )
    } else {
        Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT r.id, r.name, r.hash_crc32, r.hash_md5, r.hash_sha1, sr.source_rom_id
             FROM roms r
             LEFT JOIN source_roms sr ON sr.rom_id = r.id
             LEFT JOIN sources s ON s.id = sr.source_id AND s.source_type = 'local'
             GROUP BY r.id",
        )
    };
    let rom_rows = VerifyRomRow::find_by_statement(query).all(db).await?;

    #[allow(clippy::cast_possible_truncation)]
    let total = rom_rows.len() as u64;
    let mut stats = VerificationStats {
        verified: 0,
        unverified: 0,
        bad_dump: 0,
        not_checked: 0,
    };

    for (i, row) in rom_rows.iter().enumerate() {
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
                current_item: format!("Verifying: {}", row.name),
            });
        }

        // Compute hashes if missing and file is accessible
        let (crc, md5, sha1) = if row.hash_crc32.is_some() && row.hash_md5.is_some() && row.hash_sha1.is_some() {
            (row.hash_crc32.clone(), row.hash_md5.clone(), row.hash_sha1.clone())
        } else if let Some(ref path_str) = row.source_rom_id {
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
                    let _ = db.execute(Statement::from_sql_and_values(
                        DatabaseBackend::Sqlite,
                        "UPDATE roms SET hash_crc32 = ?, hash_md5 = ?, hash_sha1 = ? WHERE id = ?",
                        [h.crc32.clone().into(), h.md5.clone().into(), h.sha1.clone().into(), row.id.into()],
                    )).await;

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
            if row.hash_md5.is_none() && row.hash_crc32.is_none() && row.hash_sha1.is_none() {
                stats.not_checked += 1;
                continue;
            }
            (row.hash_crc32.clone(), row.hash_md5.clone(), row.hash_sha1.clone())
        };

        // Look up in dat_entries by any available hash
        let dat_match = find_dat_match(db, &crc, &md5, &sha1).await?;

        match dat_match {
            Some((entry_id, game_name, status)) => {
                let verification = if status.as_deref() == Some("baddump") {
                    stats.bad_dump += 1;
                    "bad_dump"
                } else {
                    stats.verified += 1;
                    "verified"
                };
                db.execute(Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    "UPDATE roms SET verification_status = ?, dat_entry_id = ?, dat_game_name = ? WHERE id = ?",
                    [verification.into(), entry_id.into(), game_name.into(), row.id.into()],
                )).await?;
            }
            None => {
                // Hashes computed but no DAT match
                if crc.is_some() || md5.is_some() || sha1.is_some() {
                    db.execute(Statement::from_sql_and_values(
                        DatabaseBackend::Sqlite,
                        "UPDATE roms SET verification_status = 'unverified' WHERE id = ?",
                        [row.id.into()],
                    )).await?;
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
    db: &DatabaseConnection,
    crc: &Option<String>,
    md5: &Option<String>,
    sha1: &Option<String>,
) -> AppResult<Option<(i64, String, Option<String>)>> {
    // SHA1 is most reliable
    if let Some(ref sha1_val) = sha1 {
        if let Some(model) = dat_entries::Entity::find()
            .filter(dat_entries::Column::Sha1.eq(sha1_val.as_str()))
            .one(db)
            .await?
        {
            return Ok(Some((model.id, model.game_name, model.status)));
        }
    }

    // MD5
    if let Some(ref md5_val) = md5 {
        if let Some(model) = dat_entries::Entity::find()
            .filter(dat_entries::Column::Md5.eq(md5_val.as_str()))
            .one(db)
            .await?
        {
            return Ok(Some((model.id, model.game_name, model.status)));
        }
    }

    // CRC32
    if let Some(ref crc_val) = crc {
        if let Some(model) = dat_entries::Entity::find()
            .filter(dat_entries::Column::Crc32.eq(crc_val.as_str()))
            .one(db)
            .await?
        {
            return Ok(Some((model.id, model.game_name, model.status)));
        }
    }

    Ok(None)
}

/// Get verification summary stats for a platform (or all).
pub async fn get_verification_stats(
    db: &DatabaseConnection,
    platform_id: Option<i64>,
) -> AppResult<VerificationStats> {
    let mut base = roms::Entity::find();
    if let Some(pid) = platform_id {
        base = base.filter(roms::Column::PlatformId.eq(pid));
    }

    let verified = base.clone()
        .filter(roms::Column::VerificationStatus.eq("verified"))
        .count(db).await? as i64;
    let unverified = base.clone()
        .filter(roms::Column::VerificationStatus.eq("unverified"))
        .count(db).await? as i64;
    let bad_dump = base.clone()
        .filter(roms::Column::VerificationStatus.eq("bad_dump"))
        .count(db).await? as i64;
    let not_checked = base
        .filter(roms::Column::VerificationStatus.is_null())
        .count(db).await? as i64;

    Ok(VerificationStats {
        verified,
        unverified,
        bad_dump,
        not_checked,
    })
}
