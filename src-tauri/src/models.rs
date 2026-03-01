use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub igdb_id: Option<i64>,
    pub file_extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub id: i64,
    pub name: String,
    pub source_type: SourceType,
    pub url: Option<String>,
    pub enabled: bool,
    pub last_synced_at: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Local,
    Romm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub source_id: i64,
    pub total: u64,
    pub current: u64,
    pub current_item: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomWithMeta {
    pub id: i64,
    pub platform_id: i64,
    pub platform_slug: String,
    pub platform_name: String,
    pub name: String,
    pub file_name: String,
    pub file_size: Option<i64>,
    pub regions: Vec<String>,
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub release_date: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genres: Vec<String>,
    pub themes: Vec<String>,
    pub languages: Vec<String>,
    pub cover_url: Option<String>,
    pub screenshot_urls: Vec<String>,
    pub source_id: i64,
    pub source_rom_id: Option<String>,
    pub source_type: Option<String>,
    pub retroachievements_game_id: Option<String>,
    pub wikipedia_url: Option<String>,
    pub igdb_id: Option<i64>,
    pub thegamesdb_game_id: Option<String>,
    pub favorite: bool,
    pub verification_status: Option<String>,
    pub dat_game_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformWithCount {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub rom_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub platform_count: u32,
    pub rom_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub rom_id: i64,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub status: String,
    pub error_message: Option<String>,
}

impl DownloadProgress {
    pub fn status(rom_id: i64, status: &str) -> Self {
        Self {
            rom_id,
            total_bytes: 0,
            downloaded_bytes: 0,
            status: status.to_string(),
            error_message: None,
        }
    }

    pub fn downloading(rom_id: i64, downloaded: u64, total: u64) -> Self {
        Self {
            rom_id,
            total_bytes: total,
            downloaded_bytes: downloaded,
            status: "downloading".to_string(),
            error_message: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFile {
    pub file_name: String,
    pub size: u64,
    pub last_played_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    pub total_size: u64,
    pub files: Vec<CachedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreInfo {
    pub core_name: String,
    pub core_path: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreMapping {
    pub id: i64,
    pub platform_id: i64,
    pub core_name: String,
    pub core_path: String,
    pub is_default: bool,
    pub emulator_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorDef {
    pub id: String,
    pub name: String,
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TokenPair {
    pub access_token: String,
    #[allow(dead_code)] // stored for future token refresh support
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryPage {
    pub roms: Vec<RomWithMeta>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaCredentials {
    pub username: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementData {
    pub game_title: String,
    pub num_achievements: u32,
    pub num_earned: u32,
    pub achievements: Vec<Achievement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub points: u32,
    pub badge_url: String,
    pub earned: bool,
    pub earned_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaTestResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgdbCredentials {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgdbTestResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsTestResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaveType {
    SaveFile,
    SaveState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFileInfo {
    pub file_name: String,
    pub file_path: String,
    pub save_type: SaveType,
    pub size_bytes: u64,
    pub modified_at: String,
    pub slot: Option<u32>,
    pub screenshot_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePathOverride {
    pub save_dir: Option<String>,
    pub state_dir: Option<String>,
}
