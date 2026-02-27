-- LaunchBox metadata (imported from Metadata.zip)
CREATE TABLE IF NOT EXISTS launchbox_games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    database_id TEXT NOT NULL,
    name TEXT NOT NULL,
    name_normalized TEXT NOT NULL,
    platform TEXT NOT NULL,
    overview TEXT,
    developer TEXT,
    publisher TEXT,
    genres TEXT NOT NULL DEFAULT '[]',
    release_date TEXT,
    community_rating REAL
);

CREATE INDEX IF NOT EXISTS idx_lb_games_name_norm ON launchbox_games(name_normalized, platform);
CREATE INDEX IF NOT EXISTS idx_lb_games_db_id ON launchbox_games(database_id);

CREATE TABLE IF NOT EXISTS launchbox_images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    database_id TEXT NOT NULL,
    file_name TEXT NOT NULL,
    image_type TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_lb_images_db_id_type ON launchbox_images(database_id, image_type);

-- Hasheous lookup cache (stores full API response per ROM hash)
CREATE TABLE IF NOT EXISTS hasheous_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rom_id INTEGER NOT NULL UNIQUE REFERENCES roms(id) ON DELETE CASCADE,
    hasheous_id INTEGER,
    name TEXT,
    publisher TEXT,
    year TEXT,
    description TEXT,
    genres TEXT NOT NULL DEFAULT '[]',
    igdb_game_id TEXT,
    igdb_platform_id TEXT,
    thegamesdb_game_id TEXT,
    retroachievements_game_id TEXT,
    retroachievements_platform_id TEXT,
    wikipedia_url TEXT,
    raw_response TEXT,
    fetched_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_hasheous_rom ON hasheous_cache(rom_id);
CREATE INDEX IF NOT EXISTS idx_hasheous_ra_game ON hasheous_cache(retroachievements_game_id)
    WHERE retroachievements_game_id IS NOT NULL;
