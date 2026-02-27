export interface RomWithMeta {
  id: number;
  platform_id: number;
  platform_slug: string;
  platform_name: string;
  name: string;
  file_name: string;
  file_size: number | null;
  regions: string[];
  description: string | null;
  rating: number | null;
  release_date: string | null;
  developer: string | null;
  publisher: string | null;
  genres: string[];
  themes: string[];
  languages: string[];
  cover_url: string | null;
  screenshot_urls: string[];
  source_id: number;
  source_rom_id: string | null;
  source_type: string | null;
  retroachievements_game_id: string | null;
  wikipedia_url: string | null;
  igdb_id: number | null;
  thegamesdb_game_id: string | null;
  favorite: boolean;
  verification_status: string | null;
  dat_game_name: string | null;
}

export interface PlatformWithCount {
  id: number;
  slug: string;
  name: string;
  rom_count: number;
}

export interface SourceConfig {
  id: number;
  name: string;
  source_type: "local" | "romm";
  url: string | null;
  enabled: boolean;
  last_synced_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface ScanProgress {
  source_id: number;
  total: number;
  current: number;
  current_item: string;
}

export interface DownloadProgress {
  rom_id: number;
  total_bytes: number;
  downloaded_bytes: number;
  status: "downloading" | "extracting" | "launching" | "done" | "error";
  error_message?: string;
}

export interface CoreInfo {
  core_name: string;
  core_path: string;
  display_name: string | null;
}

export interface CoreMapping {
  id: number;
  platform_id: number;
  core_name: string;
  core_path: string;
  is_default: boolean;
  emulator_type: string;
}

export interface EmulatorDef {
  id: string;
  name: string;
  platforms: string[];
}

export interface ConnectionTestResult {
  platform_count: number;
  rom_count: number;
}

export interface LibraryPage {
  roms: RomWithMeta[];
  total: number;
}

export interface MetadataProgress {
  total: number;
  current: number;
  current_item: string;
  phase: "downloading_db" | "enriching";
}

export interface AchievementData {
  game_title: string;
  num_achievements: number;
  num_earned: number;
  achievements: Achievement[];
}

export interface Achievement {
  id: number;
  title: string;
  description: string;
  points: number;
  badge_url: string;
  earned: boolean;
  earned_date: string | null;
}

export interface RaCredentials {
  username: string;
  api_key: string;
}

export interface RaTestResult {
  success: boolean;
  message: string;
}

export interface IgdbCredentials {
  client_id: string;
  client_secret: string;
}

export interface IgdbTestResult {
  success: boolean;
  message: string;
}

export interface SsCredentials {
  username: string;
  password: string;
}

export interface SsTestResult {
  success: boolean;
  message: string;
}

export interface DatDetectResult {
  detected_slug: string | null;
  header_name: string;
}

export interface DatFileInfo {
  id: number;
  name: string;
  description: string | null;
  version: string | null;
  dat_type: string;
  platform_slug: string;
  entry_count: number;
  imported_at: string;
}

export interface VerificationStats {
  verified: number;
  unverified: number;
  bad_dump: number;
  not_checked: number;
}

export interface RomSource {
  source_id: number;
  source_name: string;
  source_type: string;
  source_rom_id: string | null;
  source_url: string | null;
  file_name: string | null;
  hash_md5: string | null;
}

export type SaveType = "save_file" | "save_state";

export interface SaveFileInfo {
  file_name: string;
  file_path: string;
  save_type: SaveType;
  size_bytes: number;
  modified_at: string;
  slot: number | null;
  screenshot_path: string | null;
}

export interface SavePathOverride {
  save_dir: string | null;
  state_dir: string | null;
}
