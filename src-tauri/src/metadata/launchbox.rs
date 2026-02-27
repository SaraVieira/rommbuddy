use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

use quick_xml::events::Event;
use quick_xml::Reader;
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::ScanProgress;

/// Canonical platform slug -> `LaunchBox` platform name.
static SLUG_TO_LAUNCHBOX: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([
        ("gb", "Nintendo Game Boy"),
        ("gbc", "Nintendo Game Boy Color"),
        ("gba", "Nintendo Game Boy Advance"),
        ("nes", "Nintendo Entertainment System"),
        ("snes", "Super Nintendo Entertainment System"),
        ("n64", "Nintendo 64"),
        ("nds", "Nintendo DS"),
        ("gc", "Nintendo GameCube"),
        ("wii", "Nintendo Wii"),
        ("vb", "Nintendo Virtual Boy"),
        ("psx", "Sony Playstation"),
        ("ps2", "Sony Playstation 2"),
        ("psp", "Sony PSP"),
        ("genesis", "Sega Genesis"),
        ("gamegear", "Sega Game Gear"),
        ("mastersystem", "Sega Master System"),
        ("saturn", "Sega Saturn"),
        ("dreamcast", "Sega Dreamcast"),
        ("segacd", "Sega CD"),
        ("neogeo", "SNK Neo Geo AES"),
        ("ngp", "SNK Neo Geo Pocket"),
        ("ngpc", "SNK Neo Geo Pocket Color"),
        ("pce", "NEC TurboGrafx-16"),
        ("pcecd", "NEC TurboGrafx-CD"),
        ("lynx", "Atari Lynx"),
        ("ws", "WonderSwan"),
        ("wsc", "WonderSwan Color"),
        ("coleco", "ColecoVision"),
        ("arcade", "Arcade"),
    ])
});

/// Row returned from `launchbox_games` SQL queries.
#[derive(sqlx::FromRow)]
pub struct LaunchBoxRow {
    pub database_id: String,
    pub overview: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genres: String,
    pub release_date: Option<String>,
    pub community_rating: Option<f64>,
}

/// Normalize a game name for fuzzy matching.
pub fn normalize_for_match(name: &str) -> String {
    // 1. Strip file extension if present
    let name = match name.rsplit_once('.') {
        Some((stem, ext)) if ext.len() <= 4 && ext.chars().all(|c| c.is_ascii_alphanumeric()) => stem,
        _ => name,
    };

    // 2. Strip everything in parentheses and brackets
    let mut result = String::with_capacity(name.len());
    let mut depth_paren = 0i32;
    let mut depth_bracket = 0i32;
    for c in name.chars() {
        match c {
            '(' => depth_paren += 1,
            ')' => {
                depth_paren = (depth_paren - 1).max(0);
            }
            '[' => depth_bracket += 1,
            ']' => {
                depth_bracket = (depth_bracket - 1).max(0);
            }
            _ if depth_paren > 0 || depth_bracket > 0 => {}
            _ => result.push(c),
        }
    }

    // 3. Handle No-Intro article convention: "Name, The" -> "The Name"
    let result = move_trailing_article(&result);

    // 4. Lowercase
    let result = result.to_lowercase();

    // 5. Strip non-alphanumeric except spaces
    let result: String = result
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { ' ' })
        .collect();

    // 6. Collapse whitespace & trim
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Move trailing articles to the front: "Legend of Zelda, The" -> "The Legend of Zelda".
fn move_trailing_article(name: &str) -> String {
    if let Some(comma_pos) = name.rfind(", ") {
        let after = name[comma_pos + 2..].trim();
        let after_lower = after.to_lowercase();
        if after_lower == "the" || after_lower == "a" || after_lower == "an" {
            return format!("{after} {}", name[..comma_pos].trim());
        }
    }
    name.to_string()
}

/// Get the app data directory for caching `LaunchBox` data.
pub fn launchbox_cache_dir() -> PathBuf {
    directories::ProjectDirs::from("com", "romm-buddy", "romm-buddy")
        .map_or_else(|| PathBuf::from("."), |p| p.data_dir().to_path_buf())
        .join("launchbox")
}

/// Path to the extracted `Metadata.xml`.
pub fn metadata_xml_path() -> PathBuf {
    launchbox_cache_dir().join("Metadata.xml")
}

/// Download `Metadata.zip` and extract `Metadata.xml` to cache.
pub async fn download_and_extract(
    on_progress: impl Fn(ScanProgress) + Send,
) -> AppResult<()> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let cache_dir = launchbox_cache_dir();
    tokio::fs::create_dir_all(&cache_dir).await?;

    let url = "https://gamesdb.launchbox-app.com/Metadata.zip";
    let client = reqwest::Client::builder()
        .user_agent("romm-buddy/0.1")
        .build()
        .map_err(|e| AppError::Other(e.to_string()))?;

    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(AppError::Other(format!(
            "Failed to download LaunchBox DB: {}",
            resp.status()
        )));
    }

    let total_bytes = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    on_progress(ScanProgress {
        source_id: -1,
        total: total_bytes,
        current: 0,
        current_item: "Downloading LaunchBox database...".to_string(),
    });

    let zip_path = cache_dir.join("Metadata.zip");
    {
        let mut file = tokio::fs::File::create(&zip_path).await?;
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            #[allow(clippy::cast_possible_truncation)]
            {
                downloaded += chunk.len() as u64;
            }
            file.write_all(&chunk).await?;
            on_progress(ScanProgress {
                source_id: -1,
                total: total_bytes,
                current: downloaded,
                current_item: "Downloading LaunchBox database...".to_string(),
            });
        }
        file.flush().await?;
    }

    on_progress(ScanProgress {
        source_id: -1,
        total: 1,
        current: 0,
        current_item: "Extracting Metadata.xml...".to_string(),
    });

    // Extract Metadata.xml from zip (blocking I/O in spawn_blocking)
    let xml_path = metadata_xml_path();
    let zip_path_clone = zip_path.clone();
    tokio::task::spawn_blocking(move || -> AppResult<()> {
        let file = std::fs::File::open(&zip_path_clone)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| AppError::Other(format!("Failed to open zip: {e}")))?;

        let mut found = false;
        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| AppError::Other(format!("Failed to read zip entry: {e}")))?;
            let name = entry.name().to_string();
            if name == "Metadata.xml" || name.ends_with("/Metadata.xml") {
                let mut out = std::fs::File::create(&xml_path)?;
                std::io::copy(&mut entry, &mut out)?;
                found = true;
                break;
            }
        }

        if !found {
            return Err(AppError::Other(
                "Metadata.xml not found in LaunchBox zip".to_string(),
            ));
        }

        Ok(())
    })
    .await
    .map_err(|e| AppError::Other(format!("Task join error: {e}")))?
    ?;

    // Clean up zip
    if let Err(e) = tokio::fs::remove_file(&zip_path).await {
        log::warn!("Failed to remove LaunchBox zip file: {e}");
    }

    on_progress(ScanProgress {
        source_id: -1,
        total: 1,
        current: 1,
        current_item: "LaunchBox database ready.".to_string(),
    });

    Ok(())
}

/// Parse `Metadata.xml` and INSERT all games/images into `SQLite` tables.
/// This replaces the old in-memory index approach.
pub async fn import_to_db(
    pool: &SqlitePool,
    on_progress: impl Fn(ScanProgress) + Send + 'static,
) -> AppResult<()> {
    let xml_path = metadata_xml_path();
    if !xml_path.exists() {
        return Err(AppError::Other("Metadata.xml not found. Download the LaunchBox database first.".to_string()));
    }

    on_progress(ScanProgress {
        source_id: -1,
        total: 1,
        current: 0,
        current_item: "Clearing old LaunchBox data...".to_string(),
    });

    // Clear existing data
    sqlx::query("DELETE FROM launchbox_images").execute(pool).await?;
    sqlx::query("DELETE FROM launchbox_games").execute(pool).await?;

    on_progress(ScanProgress {
        source_id: -1,
        total: 1,
        current: 0,
        current_item: "Parsing Metadata.xml...".to_string(),
    });

    // Parse XML in a blocking task and collect games + images
    let xml_path_clone = xml_path.clone();
    let (games, images) = tokio::task::spawn_blocking(move || -> AppResult<(Vec<GameRecord>, Vec<ImageRecord>)> {
        enum Section { None, Game, GameImage }

        let file = std::fs::File::open(&xml_path_clone)?;
        let reader_buf = std::io::BufReader::with_capacity(256 * 1024, file);
        let mut reader = Reader::from_reader(reader_buf);
        reader.config_mut().trim_text(true);

        let mut games: Vec<GameRecord> = Vec::new();
        let mut images: Vec<ImageRecord> = Vec::new();
        let mut buf = Vec::with_capacity(4096);

        let mut section = Section::None;

        // Game fields
        let mut g_name = String::new();
        let mut g_platform = String::new();
        let mut g_overview: Option<String> = None;
        let mut g_developer: Option<String> = None;
        let mut g_publisher: Option<String> = None;
        let mut g_genres = String::new();
        let mut g_release_date: Option<String> = None;
        let mut g_rating: Option<f64> = None;
        let mut g_db_id = String::new();

        // Image fields
        let mut i_db_id = String::new();
        let mut i_file_name = String::new();
        let mut i_type = String::new();

        let mut current_element = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag.as_str() {
                        "Game" => {
                            section = Section::Game;
                            g_name.clear();
                            g_platform.clear();
                            g_overview = None;
                            g_developer = None;
                            g_publisher = None;
                            g_genres.clear();
                            g_release_date = None;
                            g_rating = None;
                            g_db_id.clear();
                        }
                        "GameImage" => {
                            section = Section::GameImage;
                            i_db_id.clear();
                            i_file_name.clear();
                            i_type.clear();
                        }
                        _ => {}
                    }
                    current_element = tag;
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();
                    match section {
                        Section::Game => match current_element.as_str() {
                            "Name" => g_name = text,
                            "Platform" => g_platform = text,
                            "Overview" => g_overview = Some(text),
                            "Developer" => g_developer = Some(text),
                            "Publisher" => g_publisher = Some(text),
                            "Genres" => {
                                let parsed: Vec<String> = text
                                    .split(';')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                                g_genres = serde_json::to_string(&parsed).unwrap_or_else(|_| "[]".to_string());
                            }
                            "ReleaseDate" => g_release_date = Some(text),
                            "CommunityRating" => g_rating = text.parse().ok(),
                            "DatabaseID" => g_db_id = text,
                            _ => {}
                        },
                        Section::GameImage => match current_element.as_str() {
                            "DatabaseID" => i_db_id = text,
                            "FileName" => i_file_name = text,
                            "Type" => i_type = text,
                            _ => {}
                        },
                        Section::None => {}
                    }
                }
                Ok(Event::End(e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag.as_str() {
                        "Game" => {
                            if !g_name.is_empty() && !g_db_id.is_empty() {
                                let name_normalized = normalize_for_match(&g_name);
                                games.push(GameRecord {
                                    database_id: std::mem::take(&mut g_db_id),
                                    name: std::mem::take(&mut g_name),
                                    name_normalized,
                                    platform: std::mem::take(&mut g_platform),
                                    overview: g_overview.take(),
                                    developer: g_developer.take(),
                                    publisher: g_publisher.take(),
                                    genres: if g_genres.is_empty() { "[]".to_string() } else { std::mem::take(&mut g_genres) },
                                    release_date: g_release_date.take(),
                                    community_rating: g_rating.take(),
                                });
                            }
                            section = Section::None;
                        }
                        "GameImage" => {
                            if !i_db_id.is_empty() && !i_file_name.is_empty() {
                                images.push(ImageRecord {
                                    database_id: std::mem::take(&mut i_db_id),
                                    file_name: std::mem::take(&mut i_file_name),
                                    image_type: std::mem::take(&mut i_type),
                                });
                            }
                            section = Section::None;
                        }
                        _ => {}
                    }
                    current_element.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(AppError::Other(format!("XML parse error: {e}")));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok((games, images))
    })
    .await
    .map_err(|e| AppError::Other(format!("Task join error: {e}")))?
    ?;

    #[allow(clippy::cast_possible_truncation)]
    let total_games = games.len() as u64;
    #[allow(clippy::cast_possible_truncation)]
    let total_images = images.len() as u64;

    on_progress(ScanProgress {
        source_id: -1,
        total: total_games + total_images,
        current: 0,
        current_item: format!("Importing {total_games} games..."),
    });

    // Batch insert games
    let mut count: u64 = 0;
    for chunk in games.chunks(500) {
        let mut tx = pool.begin().await?;
        for game in chunk {
            sqlx::query(
                "INSERT INTO launchbox_games (database_id, name, name_normalized, platform, overview, developer, publisher, genres, release_date, community_rating)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&game.database_id)
            .bind(&game.name)
            .bind(&game.name_normalized)
            .bind(&game.platform)
            .bind(&game.overview)
            .bind(&game.developer)
            .bind(&game.publisher)
            .bind(&game.genres)
            .bind(&game.release_date)
            .bind(game.community_rating)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        #[allow(clippy::cast_possible_truncation)]
        {
            count += chunk.len() as u64;
        }
        on_progress(ScanProgress {
            source_id: -1,
            total: total_games + total_images,
            current: count,
            current_item: format!("Imported {count}/{total_games} games..."),
        });
    }

    on_progress(ScanProgress {
        source_id: -1,
        total: total_games + total_images,
        current: total_games,
        current_item: format!("Importing {total_images} images..."),
    });

    // Batch insert images
    count = 0;
    for chunk in images.chunks(1000) {
        let mut tx = pool.begin().await?;
        for img in chunk {
            sqlx::query(
                "INSERT INTO launchbox_images (database_id, file_name, image_type)
                 VALUES (?, ?, ?)",
            )
            .bind(&img.database_id)
            .bind(&img.file_name)
            .bind(&img.image_type)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        #[allow(clippy::cast_possible_truncation)]
        {
            count += chunk.len() as u64;
        }
        on_progress(ScanProgress {
            source_id: -1,
            total: total_games + total_images,
            current: total_games + count,
            current_item: format!("Imported {count}/{total_images} images..."),
        });
    }

    // Clean up Metadata.xml after import
    if let Err(e) = tokio::fs::remove_file(&xml_path).await {
        log::warn!("Failed to remove Metadata.xml after import: {e}");
    }

    on_progress(ScanProgress {
        source_id: -1,
        total: 1,
        current: 1,
        current_item: "LaunchBox import complete.".to_string(),
    });

    Ok(())
}

/// Check if `launchbox_games` table has data.
pub async fn has_imported_db(pool: &SqlitePool) -> bool {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM launchbox_games")
        .fetch_one(pool)
        .await
        .is_ok_and(|c| c > 0)
}

/// Look up a game by name and platform slug from the `SQLite` tables.
pub async fn find_by_name(
    pool: &SqlitePool,
    game_name: &str,
    platform_slug: &str,
) -> Option<LaunchBoxRow> {
    let lb_platform = SLUG_TO_LAUNCHBOX.get(platform_slug)?;
    let normalized = normalize_for_match(game_name);

    let sql = "SELECT database_id, overview, developer, publisher, genres, release_date, community_rating
         FROM launchbox_games
         WHERE name_normalized = ? AND platform = ?
         LIMIT 1";

    // Try exact normalized match
    let row = sqlx::query_as::<_, LaunchBoxRow>(sql)
        .bind(&normalized)
        .bind(lb_platform)
        .fetch_optional(pool)
        .await
        .ok()?;

    if row.is_some() {
        return row;
    }

    // Try with collapsed subtitles (" - " -> " ")
    let no_dash = normalized.replace(" - ", " ");
    if no_dash != normalized {
        let row = sqlx::query_as::<_, LaunchBoxRow>(sql)
            .bind(&no_dash)
            .bind(lb_platform)
            .fetch_optional(pool)
            .await
            .ok()?;

        if row.is_some() {
            return row;
        }
    }

    None
}

/// Get the best cover image URL for a `LaunchBox` `database_id`.
pub async fn get_image_url(pool: &SqlitePool, database_id: &str) -> Option<String> {
    let file_name = sqlx::query_scalar::<_, String>(
        "SELECT file_name FROM launchbox_images
         WHERE database_id = ?
         ORDER BY (image_type = 'Box - Front') DESC
         LIMIT 1",
    )
    .bind(database_id)
    .fetch_optional(pool)
    .await
    .ok()??;

    Some(format!("https://images.launchbox-app.com/{file_name}"))
}

/// Get screenshot image URLs for a `LaunchBox` `database_id`.
pub async fn get_screenshot_urls(pool: &SqlitePool, database_id: &str) -> Vec<String> {
    let file_names = sqlx::query_scalar::<_, String>(
        "SELECT file_name FROM launchbox_images
         WHERE database_id = ? AND image_type LIKE '%Screenshot%'
         LIMIT 10",
    )
    .bind(database_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    file_names
        .into_iter()
        .map(|f| format!("https://images.launchbox-app.com/{f}"))
        .collect()
}

struct GameRecord {
    database_id: String,
    name: String,
    name_normalized: String,
    platform: String,
    overview: Option<String>,
    developer: Option<String>,
    publisher: Option<String>,
    genres: String,
    release_date: Option<String>,
    community_rating: Option<f64>,
}

struct ImageRecord {
    database_id: String,
    file_name: String,
    image_type: String,
}
