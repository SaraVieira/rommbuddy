-- WAL mode is set in db.rs via SqliteConnectOptions, not here.

-- Canonical system/platform list
CREATE TABLE IF NOT EXISTS platforms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    igdb_id INTEGER,
    screenscraper_id INTEGER,
    file_extensions TEXT NOT NULL DEFAULT '[]',
    folder_aliases TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source_type TEXT NOT NULL CHECK (source_type IN ('local', 'romm', 'myrient')),
    url TEXT,
    credentials TEXT NOT NULL DEFAULT '{}',
    settings TEXT NOT NULL DEFAULT '{}',
    enabled INTEGER NOT NULL DEFAULT 1,
    last_synced_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS roms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    platform_id INTEGER NOT NULL REFERENCES platforms(id),
    name TEXT NOT NULL,
    file_name TEXT NOT NULL,
    file_size INTEGER,
    hash_crc32 TEXT,
    hash_md5 TEXT,
    hash_sha1 TEXT,
    regions TEXT NOT NULL DEFAULT '[]',
    languages TEXT NOT NULL DEFAULT '[]',
    verification_status TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS source_roms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rom_id INTEGER NOT NULL REFERENCES roms(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    source_rom_id TEXT,
    source_url TEXT,
    source_meta TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(rom_id, source_id)
);

CREATE TABLE IF NOT EXISTS metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rom_id INTEGER NOT NULL UNIQUE REFERENCES roms(id) ON DELETE CASCADE,
    igdb_id INTEGER,
    screenscraper_id INTEGER,
    description TEXT,
    rating REAL,
    release_date TEXT,
    developer TEXT,
    publisher TEXT,
    genres TEXT NOT NULL DEFAULT '[]',
    themes TEXT NOT NULL DEFAULT '[]',
    metadata_fetched_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS artwork (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rom_id INTEGER NOT NULL REFERENCES roms(id) ON DELETE CASCADE,
    art_type TEXT NOT NULL CHECK (art_type IN ('cover', 'screenshot', 'fanart', 'banner', 'logo')),
    url TEXT,
    local_path TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS library (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rom_id INTEGER NOT NULL REFERENCES roms(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    file_path TEXT,
    play_count INTEGER NOT NULL DEFAULT 0,
    last_played_at TEXT,
    favorite INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(rom_id, source_id)
);

CREATE TABLE IF NOT EXISTS downloads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rom_id INTEGER NOT NULL REFERENCES roms(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'downloading', 'completed', 'failed', 'cancelled')),
    progress REAL NOT NULL DEFAULT 0.0,
    file_path TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS core_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    platform_id INTEGER NOT NULL REFERENCES platforms(id) ON DELETE CASCADE,
    core_name TEXT NOT NULL,
    core_path TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE VIRTUAL TABLE IF NOT EXISTS roms_fts USING fts5(
    name,
    file_name,
    content='roms',
    content_rowid='id'
);

CREATE TRIGGER IF NOT EXISTS roms_ai AFTER INSERT ON roms BEGIN
    INSERT INTO roms_fts(rowid, name, file_name) VALUES (new.id, new.name, new.file_name);
END;

CREATE TRIGGER IF NOT EXISTS roms_ad AFTER DELETE ON roms BEGIN
    INSERT INTO roms_fts(roms_fts, rowid, name, file_name) VALUES ('delete', old.id, old.name, old.file_name);
END;

CREATE TRIGGER IF NOT EXISTS roms_au AFTER UPDATE ON roms BEGIN
    INSERT INTO roms_fts(roms_fts, rowid, name, file_name) VALUES ('delete', old.id, old.name, old.file_name);
    INSERT INTO roms_fts(rowid, name, file_name) VALUES (new.id, new.name, new.file_name);
END;

CREATE INDEX IF NOT EXISTS idx_roms_platform_name ON roms(platform_id, name);
CREATE INDEX IF NOT EXISTS idx_roms_hash_sha1 ON roms(hash_sha1) WHERE hash_sha1 IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_roms_hash_crc32 ON roms(hash_crc32) WHERE hash_crc32 IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_source_roms_source ON source_roms(source_id);
CREATE INDEX IF NOT EXISTS idx_source_roms_rom ON source_roms(rom_id);
CREATE INDEX IF NOT EXISTS idx_library_platform ON library(rom_id);
CREATE INDEX IF NOT EXISTS idx_artwork_rom ON artwork(rom_id);
CREATE INDEX IF NOT EXISTS idx_downloads_status ON downloads(status);
CREATE INDEX IF NOT EXISTS idx_core_mappings_platform ON core_mappings(platform_id);

INSERT OR IGNORE INTO platforms (slug, name, file_extensions, folder_aliases) VALUES
    ('gb', 'Game Boy', '["gb"]', '["gb","GB"]'),
    ('gbc', 'Game Boy Color', '["gbc"]', '["gbc","GBC"]'),
    ('gba', 'Game Boy Advance', '["gba"]', '["gba","GBA"]'),
    ('nes', 'NES / Famicom', '["nes","unf","unif"]', '["nes","FC","fc"]'),
    ('snes', 'SNES / Super Famicom', '["sfc","smc"]', '["snes","sfc","SFC","smc"]'),
    ('n64', 'Nintendo 64', '["n64","z64","v64"]', '["n64","N64"]'),
    ('nds', 'Nintendo DS', '["nds"]', '["nds","NDS"]'),
    ('psx', 'PlayStation', '["bin","cue","iso","chd","m3u"]', '["psx","ps1","PS","ps"]'),
    ('ps2', 'PlayStation 2', '["iso","chd","bin","cue"]', '["ps2","PS2"]'),
    ('psp', 'PlayStation Portable', '["iso","cso"]', '["psp","PSP"]'),
    ('genesis', 'Sega Genesis / Mega Drive', '["md","gen","smd","bin"]', '["genesis","megadrive","MD","md","gen"]'),
    ('segacd', 'Sega CD', '["bin","cue","chd","iso"]', '["segacd","SEGACD","scd"]'),
    ('saturn', 'Sega Saturn', '["bin","cue","chd","iso"]', '["saturn","SATURN"]'),
    ('dreamcast', 'Dreamcast', '["chd","gdi","cdi"]', '["dreamcast","DREAMCAST","dc"]'),
    ('gamegear', 'Game Gear', '["gg"]', '["gamegear","gg","GG"]'),
    ('mastersystem', 'Master System', '["sms"]', '["mastersystem","sms","MS","ms"]'),
    ('neogeo', 'Neo Geo', '["zip"]', '["neogeo","NEOGEO"]'),
    ('arcade', 'Arcade (MAME/FBNeo)', '["zip"]', '["arcade","ARCADE","mame","fbneo"]'),
    ('pce', 'TurboGrafx-16', '["pce"]', '["pcengine","pce","PCE"]'),
    ('pcecd', 'TurboGrafx CD', '["chd","cue","bin"]', '["pcenginecd","pcecd","PCECD"]'),
    ('ngp', 'Neo Geo Pocket', '["ngp","ngc"]', '["ngp","NGP"]'),
    ('ws', 'WonderSwan', '["ws","wsc"]', '["wonderswan","ws","WS"]'),
    ('lynx', 'Atari Lynx', '["lnx"]', '["lynx","LYNX"]'),
    ('vb', 'Virtual Boy', '["vb"]', '["virtualboy","vb","VB"]');
