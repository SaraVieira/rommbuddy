CREATE TABLE IF NOT EXISTS igdb_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rom_id INTEGER NOT NULL UNIQUE REFERENCES roms(id) ON DELETE CASCADE,
    igdb_id INTEGER,
    name TEXT,
    summary TEXT,
    storyline TEXT,
    aggregated_rating REAL,
    first_release_date TEXT,
    genres TEXT,          -- JSON array
    themes TEXT,          -- JSON array
    game_modes TEXT,      -- JSON array
    player_perspectives TEXT, -- JSON array
    developer TEXT,
    publisher TEXT,
    cover_image_id TEXT,
    screenshot_image_ids TEXT, -- JSON array
    franchise_name TEXT,
    raw_response TEXT,
    fetched_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_igdb_cache_rom_id ON igdb_cache(rom_id);
