-- DAT file metadata (imported from No-Intro / Redump XML files)
CREATE TABLE IF NOT EXISTS dat_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    version TEXT,
    dat_type TEXT NOT NULL DEFAULT 'no-intro', -- 'no-intro' or 'redump'
    platform_slug TEXT NOT NULL,
    entry_count INTEGER NOT NULL DEFAULT 0,
    imported_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_dat_files_platform ON dat_files(platform_slug);

-- Individual ROM entries from DAT files
CREATE TABLE IF NOT EXISTS dat_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    dat_file_id INTEGER NOT NULL REFERENCES dat_files(id) ON DELETE CASCADE,
    game_name TEXT NOT NULL,
    rom_name TEXT NOT NULL,
    size INTEGER,
    crc32 TEXT,
    md5 TEXT,
    sha1 TEXT,
    status TEXT -- NULL = good, 'baddump', 'nodump', 'verified'
);

CREATE INDEX IF NOT EXISTS idx_dat_entries_crc32 ON dat_entries(crc32) WHERE crc32 IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_dat_entries_md5 ON dat_entries(md5) WHERE md5 IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_dat_entries_sha1 ON dat_entries(sha1) WHERE sha1 IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_dat_entries_file ON dat_entries(dat_file_id);

-- Add DAT reference columns to roms table
ALTER TABLE roms ADD COLUMN dat_entry_id INTEGER REFERENCES dat_entries(id) ON DELETE SET NULL;
ALTER TABLE roms ADD COLUMN dat_game_name TEXT;
