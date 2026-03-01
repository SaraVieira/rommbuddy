use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;

use chrono::{DateTime, Utc};

use crate::models::{SaveFileInfo, SaveType};

/// Default save and state directories for a given emulator.
pub struct EmulatorSavePaths {
    pub save_dirs: Vec<String>,
    pub state_dirs: Vec<String>,
}

/// Read RetroArch's retroarch.cfg to get the actual configured save/state directories.
/// Falls back to Application Support defaults if config can't be read.
fn read_retroarch_config_dirs() -> (Vec<String>, Vec<String>) {
    let home = dirs::home_dir().unwrap_or_default();
    let app_support = home.join("Library/Application Support");

    let default_saves = app_support
        .join("RetroArch/saves")
        .to_string_lossy()
        .into_owned();
    let default_states = app_support
        .join("RetroArch/states")
        .to_string_lossy()
        .into_owned();

    // Try to read RetroArch config
    let cfg_path = app_support.join("RetroArch/config/retroarch.cfg");
    let cfg_path_alt = app_support.join("RetroArch/retroarch.cfg");

    let cfg = if cfg_path.exists() {
        cfg_path
    } else if cfg_path_alt.exists() {
        cfg_path_alt
    } else {
        return (vec![default_saves], vec![default_states]);
    };

    let file = match std::fs::File::open(&cfg) {
        Ok(f) => f,
        Err(_) => return (vec![default_saves], vec![default_states]),
    };

    let mut save_dir = None;
    let mut state_dir = None;
    let reader = std::io::BufReader::new(file);

    for line in reader.lines().map_while(Result::ok) {
        let line = line.trim().to_string();
        if let Some(val) = parse_retroarch_cfg_value(&line, "savefile_directory") {
            save_dir = Some(expand_tilde(&val));
        } else if let Some(val) = parse_retroarch_cfg_value(&line, "savestate_directory") {
            state_dir = Some(expand_tilde(&val));
        }
        if save_dir.is_some() && state_dir.is_some() {
            break;
        }
    }

    (
        vec![save_dir.unwrap_or(default_saves)],
        vec![state_dir.unwrap_or(default_states)],
    )
}

/// Parse a key = "value" line from retroarch.cfg
fn parse_retroarch_cfg_value(line: &str, key: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with(key) {
        return None;
    }
    let rest = trimmed[key.len()..].trim();
    let rest = rest.strip_prefix('=')?;
    let rest = rest.trim();
    let rest = rest.trim_matches('"');
    if rest.is_empty() {
        return None;
    }
    Some(rest.to_string())
}

/// Expand ~ to home directory
fn expand_tilde(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped).to_string_lossy().into_owned();
        }
    }
    path.to_string()
}

/// Returns a map of emulator id -> default macOS save/state paths.
pub fn default_save_paths() -> HashMap<&'static str, EmulatorSavePaths> {
    let home = dirs::home_dir().unwrap_or_default();
    let app_support = home.join("Library/Application Support");

    let mut map = HashMap::new();

    // RetroArch: read actual config to get real directories
    let (ra_saves, ra_states) = read_retroarch_config_dirs();
    map.insert(
        "retroarch",
        EmulatorSavePaths {
            save_dirs: ra_saves,
            state_dirs: ra_states,
        },
    );

    map.insert(
        "dolphin",
        EmulatorSavePaths {
            save_dirs: vec![
                app_support
                    .join("Dolphin/GC")
                    .to_string_lossy()
                    .into_owned(),
                app_support
                    .join("Dolphin/Wii")
                    .to_string_lossy()
                    .into_owned(),
            ],
            state_dirs: vec![app_support
                .join("Dolphin/StateSaves")
                .to_string_lossy()
                .into_owned()],
        },
    );

    map.insert(
        "duckstation",
        EmulatorSavePaths {
            save_dirs: vec![app_support
                .join("DuckStation/memcards")
                .to_string_lossy()
                .into_owned()],
            state_dirs: vec![app_support
                .join("DuckStation/savestates")
                .to_string_lossy()
                .into_owned()],
        },
    );

    map.insert(
        "pcsx2",
        EmulatorSavePaths {
            save_dirs: vec![app_support
                .join("PCSX2/memcards")
                .to_string_lossy()
                .into_owned()],
            state_dirs: vec![app_support
                .join("PCSX2/sstates")
                .to_string_lossy()
                .into_owned()],
        },
    );

    map.insert(
        "mgba",
        EmulatorSavePaths {
            save_dirs: vec![app_support
                .join("mGBA/saves")
                .to_string_lossy()
                .into_owned()],
            state_dirs: vec![app_support
                .join("mGBA/states")
                .to_string_lossy()
                .into_owned()],
        },
    );

    map.insert(
        "melonds",
        EmulatorSavePaths {
            save_dirs: vec![app_support
                .join("melonDS/saves")
                .to_string_lossy()
                .into_owned()],
            state_dirs: vec![app_support
                .join("melonDS/states")
                .to_string_lossy()
                .into_owned()],
        },
    );

    map.insert(
        "cemu",
        EmulatorSavePaths {
            save_dirs: vec![app_support
                .join("Cemu/mlc01/usr/save")
                .to_string_lossy()
                .into_owned()],
            state_dirs: vec![],
        },
    );

    map.insert(
        "xemu",
        EmulatorSavePaths {
            save_dirs: vec![app_support
                .join("xemu")
                .to_string_lossy()
                .into_owned()],
            state_dirs: vec![],
        },
    );

    map.insert(
        "rpcs3",
        EmulatorSavePaths {
            save_dirs: vec![app_support
                .join("rpcs3/dev_hdd0/home/00000001/savedata")
                .to_string_lossy()
                .into_owned()],
            state_dirs: vec![app_support
                .join("rpcs3/savestates")
                .to_string_lossy()
                .into_owned()],
        },
    );

    map
}

/// Classify a file extension as either a save file or save state.
pub fn classify_extension(ext: &str) -> Option<SaveType> {
    let ext_lower = ext.to_lowercase();

    // Save file extensions
    match ext_lower.as_str() {
        "sav" | "srm" | "eep" | "fla" | "mcr" | "mcd" | "ps2" | "bin" => {
            return Some(SaveType::SaveFile);
        }
        _ => {}
    }

    // Save state extensions
    match ext_lower.as_str() {
        "state" | "undo" | "oops" | "p2s" => {
            return Some(SaveType::SaveState);
        }
        _ => {}
    }

    // state0-state99
    if let Some(rest) = ext_lower.strip_prefix("state") {
        if let Ok(n) = rest.parse::<u32>() {
            if n <= 99 {
                return Some(SaveType::SaveState);
            }
        }
    }

    // ss0-ss9 (mGBA)
    if let Some(rest) = ext_lower.strip_prefix("ss") {
        if let Ok(n) = rest.parse::<u32>() {
            if n <= 9 {
                return Some(SaveType::SaveState);
            }
        }
    }

    // s01-s99 (Dolphin)
    if let Some(rest) = ext_lower.strip_prefix('s') {
        if let Ok(n) = rest.parse::<u32>() {
            if (1..=99).contains(&n) {
                return Some(SaveType::SaveState);
            }
        }
    }

    None
}

/// Extract a slot number from a save state extension (e.g., "state3" -> 3, "state" -> 0, "ss1" -> 1, "s01" -> 1).
pub fn extract_slot(ext: &str) -> Option<u32> {
    let ext_lower = ext.to_lowercase();

    // state / state0-state99 (RetroArch)
    if let Some(rest) = ext_lower.strip_prefix("state") {
        if rest.is_empty() {
            return Some(0);
        }
        if let Ok(n) = rest.parse::<u32>() {
            return Some(n);
        }
    }

    // ss / ss0-ss9 (mGBA)
    if let Some(rest) = ext_lower.strip_prefix("ss") {
        if rest.is_empty() {
            return Some(0);
        }
        if let Ok(n) = rest.parse::<u32>() {
            return Some(n);
        }
    }

    // s01-s99 (Dolphin)
    if let Some(rest) = ext_lower.strip_prefix('s') {
        if let Ok(n) = rest.parse::<u32>() {
            return Some(n);
        }
    }

    // p2s (PCSX2) — single format, no slot number
    if ext_lower == "p2s" {
        return Some(0);
    }

    None
}

/// Scan directories for save files matching the given ROM file name.
///
/// Matches files whose stem exactly matches the ROM's file stem (without extension).
/// Returns results sorted by `modified_at` descending (newest first).
pub fn scan_for_saves(
    rom_file_name: &str,
    save_dirs: &[String],
    state_dirs: &[String],
) -> Vec<SaveFileInfo> {
    let rom_stem = Path::new(rom_file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if rom_stem.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();

    let scan_dir = |dir_path: &str, results: &mut Vec<SaveFileInfo>| {
        let dir = Path::new(dir_path);
        if !dir.is_dir() {
            return;
        }

        // Collect all files: top-level + one level of subdirectories
        // (RetroArch organizes saves in subdirs per core, e.g., states/Stella/)
        let mut files_to_check = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    files_to_check.push(path);
                } else if path.is_dir() {
                    // Scan one level deep into subdirectories
                    if let Ok(sub_entries) = std::fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if sub_path.is_file() {
                                files_to_check.push(sub_path);
                            }
                        }
                    }
                }
            }
        }

        for path in &files_to_check {
            if !path.is_file() {
                continue;
            }

            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            let file_stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_lowercase(),
                None => continue,
            };

            // Check if this file's stem matches the ROM stem
            if file_stem != rom_stem {
                continue;
            }

            let ext = match path.extension().and_then(|e| e.to_str()) {
                Some(e) => e,
                None => continue,
            };

            let save_type = match classify_extension(ext) {
                Some(t) => t,
                None => continue,
            };

            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size_bytes = metadata.len();

            let modified_at = metadata
                .modified()
                .ok()
                .map(|t| {
                    let dt: DateTime<Utc> = t.into();
                    dt.to_rfc3339()
                })
                .unwrap_or_default();

            let slot = extract_slot(ext);

            // Look for a screenshot with the same base name
            let screenshot_path = {
                let ss_png = path.with_extension(format!("{ext}.png"));
                let ss_plain = path.with_extension("png");
                if ss_png.is_file() {
                    Some(ss_png.to_string_lossy().into_owned())
                } else if ss_plain.is_file() {
                    Some(ss_plain.to_string_lossy().into_owned())
                } else {
                    None
                }
            };

            results.push(SaveFileInfo {
                file_name,
                file_path: path.to_string_lossy().into_owned(),
                save_type,
                size_bytes,
                modified_at,
                slot,
                screenshot_path,
            });
        }
    };

    for dir in save_dirs {
        scan_dir(dir, &mut results);
    }
    for dir in state_dirs {
        scan_dir(dir, &mut results);
    }

    // Deduplicate by file_path (same file can be found from multiple scan dirs)
    results.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    results.dedup_by(|a, b| a.file_path == b.file_path);

    // Sort by modified_at descending (newest first)
    results.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    results
}
