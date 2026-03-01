use std::path::Path;

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseBackend,
    DatabaseConnection, EntityTrait, QueryFilter, Statement,
};
use tokio_util::sync::CancellationToken;

use crate::dedup;
use crate::error::AppResult;
use crate::models::ScanProgress;
use crate::platform_registry;

/// Known ROM file extensions -- files matching these are indexed.
const ROM_EXTENSIONS: &[&str] = &[
    "gb", "gbc", "gba", "nes", "sfc", "smc", "n64", "z64", "v64",
    "nds", "3ds", "iso", "bin", "cue", "chd", "rvz", "wbfs", "rom",
    "md", "gen", "smd", "gg", "sms", "pce", "ngp", "ngc",
    "ws", "wsc", "lnx", "vb", "zip", "7z", "m3u",
    "a26", "a78", "col", "sg", "int", "jag",
    "psx", "pbp", "cso", "xci", "nsp",
];

/// Detected folder layout convention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolderLayout {
    /// Lowercase slugs: `gb/`, `gba/`, `snes/` -- ES-DE, `RetroPie`, `ArkOS`, `EmuDeck`.
    EsDe,
    /// `roms/` subdirectory containing lowercase slugs -- Batocera, KNULLI.
    Batocera,
    /// `ROMS/` + `MUOS/` sibling directories.
    MuOs,
    /// "Name (TAG)/" pattern -- `MinUI`.
    MinUi,
    /// `ALL_CAPS` folder names -- `OnionOS`.
    OnionOs,
    /// Could not detect layout; treat folder names as lowercase slugs.
    Unknown,
}

/// Detect the folder layout convention of a ROM directory.
pub fn detect_layout(root: &Path) -> FolderLayout {
    let entries: Vec<String> = match std::fs::read_dir(root) {
        Ok(rd) => rd
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect(),
        Err(e) => {
            log::warn!("Failed to read directory {}: {e}", root.display());
            return FolderLayout::Unknown;
        }
    };

    if entries.is_empty() {
        return FolderLayout::Unknown;
    }

    // `MuOS`: has both `ROMS/` and `MUOS/` directories
    if entries.iter().any(|n| n == "ROMS") && entries.iter().any(|n| n == "MUOS") {
        return FolderLayout::MuOs;
    }

    // Batocera/KNULLI/`ArkOS`: has a `roms/` or `EASYROMS/` subdirectory
    let batocera_dir = if entries.iter().any(|n| n == "roms") {
        Some(root.join("roms"))
    } else if entries.iter().any(|n| n == "EASYROMS") {
        Some(root.join("EASYROMS"))
    } else {
        None
    };
    if let Some(roms_sub) = batocera_dir {
        if let Ok(sub_entries) = std::fs::read_dir(&roms_sub) {
            let sub_names: Vec<String> = sub_entries
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            let known_count = sub_names
                .iter()
                .filter(|n| platform_registry::is_known_folder(&n.to_lowercase()))
                .count();
            if known_count >= 2 {
                return FolderLayout::Batocera;
            }
        }
    }

    // `MinUI`: folders matching "Anything (TAG)" pattern
    let minui_count = entries
        .iter()
        .filter(|n| {
            n.contains('(')
                && n.ends_with(')')
                && n.rfind('(').is_some_and(|i| i > 0)
        })
        .count();
    if minui_count >= 3 {
        return FolderLayout::MinUi;
    }

    // `OnionOS`: all-uppercase folder names matching known set
    let upper_count = entries
        .iter()
        .filter(|n| {
            !n.is_empty()
                && n.chars()
                    .all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '_')
        })
        .count();
    if upper_count > entries.len() / 2 && upper_count >= 3 {
        return FolderLayout::OnionOs;
    }

    // ES-DE / `ArkOS`: lowercase slug folders matching known platforms
    let esde_count = entries
        .iter()
        .filter(|n| platform_registry::is_known_folder(n.as_str()))
        .count();
    if esde_count >= 3 {
        return FolderLayout::EsDe;
    }

    FolderLayout::Unknown
}

/// Extract the `MinUI` tag from a folder name like "Game Boy Advance (GBA)" -> "GBA".
fn extract_minui_tag(folder_name: &str) -> Option<&str> {
    let open = folder_name.rfind('(')?;
    let close = folder_name.rfind(')')?;
    if close > open + 1 && close == folder_name.len() - 1 {
        Some(&folder_name[open + 1..close])
    } else {
        None
    }
}

/// Resolve a folder name to a canonical platform slug using the detected layout.
fn resolve_folder_to_slug(folder_name: &str, layout: &FolderLayout) -> Option<String> {
    if layout == &FolderLayout::MinUi {
        let tag = extract_minui_tag(folder_name)?;
        let lower = tag.to_lowercase();
        platform_registry::resolve_folder(&lower).map(|s| s.to_string())
    } else {
        let lower = folder_name.to_lowercase();
        let normalized = lower.replace(['-', '_'], "");
        if let Some(slug) = platform_registry::resolve_folder(&lower) {
            Some(slug.to_string())
        } else {
            platform_registry::resolve_folder(&normalized).map(|s| s.to_string())
        }
    }
}

/// Check if a file has a ROM extension.
fn is_rom_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| ROM_EXTENSIONS.contains(&e.to_lowercase().as_str()))
}

/// Get the actual root for ROM folders depending on layout.
fn get_roms_root(root: &Path, layout: &FolderLayout) -> std::path::PathBuf {
    match layout {
        FolderLayout::Batocera => {
            let roms = root.join("roms");
            if roms.exists() { roms } else { root.join("EASYROMS") }
        }
        FolderLayout::MuOs => root.join("ROMS"),
        _ => root.to_path_buf(),
    }
}

/// Count total ROM files for progress reporting.
fn count_rom_files(roms_root: &Path) -> u64 {
    let mut count: u64 = 0;
    if let Ok(dirs) = std::fs::read_dir(roms_root) {
        for entry in dirs.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(files) = std::fs::read_dir(&path) {
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        count += files
                            .filter_map(std::result::Result::ok)
                            .filter(|f| is_rom_file(&f.path()))
                            .count() as u64;
                    }
                }
            }
        }
    }
    count
}

/// Test a local path: detect layout and count platforms/ROMs.
pub fn test_local_path(root: &Path) -> AppResult<(FolderLayout, u32, u64)> {
    if !root.exists() || !root.is_dir() {
        return Err(crate::error::AppError::Other(format!(
            "Path does not exist or is not a directory: {}",
            root.display()
        )));
    }

    let layout = detect_layout(root);
    let roms_root = get_roms_root(root, &layout);

    let mut platform_count: u32 = 0;
    let mut rom_count: u64 = 0;

    if let Ok(dirs) = std::fs::read_dir(&roms_root) {
        for entry in dirs.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let folder_name = entry.file_name().to_string_lossy().into_owned();
            if resolve_folder_to_slug(&folder_name, &layout).is_some() {
                #[allow(clippy::cast_possible_truncation)]
                let file_count = std::fs::read_dir(&path)
                    .map(|rd| {
                        rd.filter_map(std::result::Result::ok)
                            .filter(|f| is_rom_file(&f.path()))
                            .count() as u64
                    })
                    .unwrap_or(0);
                if file_count > 0 {
                    platform_count += 1;
                    rom_count += file_count;
                }
            }
        }
    }

    Ok((layout, platform_count, rom_count))
}

/// Scanned ROM file info collected from the filesystem.
struct ScannedRomFile {
    canonical_slug: String,
    file_path: std::path::PathBuf,
    file_name: String,
    rom_name: String,
    file_size: Option<i64>,
}

/// Scan the filesystem for ROM files, returning structured results.
/// This is a blocking function that should be called from `spawn_blocking`.
fn scan_local_rom_files(
    root: &Path,
) -> AppResult<(Vec<ScannedRomFile>, u64)> {
    let layout = detect_layout(root);
    let roms_root = get_roms_root(root, &layout);
    let total_roms = count_rom_files(&roms_root);

    let mut dirs: Vec<_> = std::fs::read_dir(&roms_root)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .collect();
    dirs.sort_by_key(std::fs::DirEntry::file_name);

    let mut results = Vec::new();

    for dir_entry in dirs {
        let folder_name = dir_entry.file_name().to_string_lossy().into_owned();
        let Some(canonical_slug) = resolve_folder_to_slug(&folder_name, &layout) else {
            continue;
        };

        let mut files: Vec<_> = std::fs::read_dir(dir_entry.path())?
            .filter_map(std::result::Result::ok)
            .filter(|e| is_rom_file(&e.path()))
            .collect();
        files.sort_by_key(std::fs::DirEntry::file_name);

        for file_entry in files {
            let file_path = file_entry.path();
            let file_name = file_entry.file_name().to_string_lossy().into_owned();
            let rom_name = file_path
                .file_stem()
                .map_or_else(|| file_name.clone(), |s| s.to_string_lossy().into_owned());

            #[allow(clippy::cast_possible_wrap)]
            let file_size = file_entry.metadata().map(|m| m.len() as i64).ok();

            results.push(ScannedRomFile {
                canonical_slug: canonical_slug.clone(),
                file_path,
                file_name,
                rom_name,
                file_size,
            });
        }
    }

    Ok((results, total_roms))
}

/// Sync a local filesystem source into the database.
pub async fn sync_local_to_db(
    source_id: i64,
    root: &Path,
    db: &DatabaseConnection,
    on_progress: impl Fn(ScanProgress) + Send,
    cancel: CancellationToken,
) -> AppResult<()> {
    // Scan the filesystem in a blocking task to avoid stalling the async runtime
    let root_owned = root.to_path_buf();
    let (scanned_files, total_roms) = tokio::task::spawn_blocking(move || {
        scan_local_rom_files(&root_owned)
    })
    .await
    .map_err(|e| crate::error::AppError::Other(format!("Task join error: {e}")))??;

    // Cache platform IDs to avoid repeated lookups
    let mut platform_cache: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

    for (idx, scanned) in scanned_files.iter().enumerate() {
        if cancel.is_cancelled() {
            return Ok(());
        }

        // Find or create platform (with local cache)
        let local_platform_id = if let Some(&id) = platform_cache.get(&scanned.canonical_slug) {
            id
        } else {
            use crate::entity::platforms;
            let existing = platforms::Entity::find()
                .filter(platforms::Column::Slug.eq(&scanned.canonical_slug))
                .one(db)
                .await?;
            let id = if let Some(p) = existing {
                p.id
            } else {
                let display_name = platform_registry::display_name(&scanned.canonical_slug)
                    .unwrap_or(scanned.canonical_slug.as_str());
                log::info!(
                    "Creating new platform: slug='{}', name='{display_name}'",
                    scanned.canonical_slug,
                );
                let model = platforms::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    slug: Set(scanned.canonical_slug.clone()),
                    name: Set(display_name.to_string()),
                    igdb_id: Set(None),
                    screenscraper_id: Set(platform_registry::ss_id(&scanned.canonical_slug).map(|id| id as i64)),
                    file_extensions: Set("[]".to_string()),
                    folder_aliases: Set("[]".to_string()),
                    created_at: Set(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
                    updated_at: Set(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
                }.insert(db).await?;
                model.id
            };
            platform_cache.insert(scanned.canonical_slug.clone(), id);
            id
        };

        #[allow(clippy::cast_possible_truncation)]
        let current = (idx as u64) + 1;
        on_progress(ScanProgress {
            source_id,
            total: total_roms,
            current,
            current_item: scanned.rom_name.clone(),
        });

        let abs_path = scanned.file_path.to_string_lossy().into_owned();
        let _rom_id = dedup::upsert_rom_deduped(
            db,
            local_platform_id,
            &scanned.rom_name,
            &scanned.file_name,
            scanned.file_size,
            "[]",
            None,
            source_id,
            Some(&abs_path),
            None,
        )
        .await?;
    }

    // Update source last_synced_at
    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "UPDATE sources SET last_synced_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?",
        [source_id.into()],
    )).await?;

    Ok(())
}
