-- Populate screenscraper_id on platforms table
UPDATE platforms SET screenscraper_id = 9 WHERE slug = 'gb';
UPDATE platforms SET screenscraper_id = 10 WHERE slug = 'gbc';
UPDATE platforms SET screenscraper_id = 12 WHERE slug = 'gba';
UPDATE platforms SET screenscraper_id = 3 WHERE slug = 'nes';
UPDATE platforms SET screenscraper_id = 4 WHERE slug = 'snes';
UPDATE platforms SET screenscraper_id = 14 WHERE slug = 'n64';
UPDATE platforms SET screenscraper_id = 15 WHERE slug = 'nds';
UPDATE platforms SET screenscraper_id = 57 WHERE slug = 'psx';
UPDATE platforms SET screenscraper_id = 1 WHERE slug = 'genesis';
UPDATE platforms SET screenscraper_id = 75 WHERE slug = 'arcade';
UPDATE platforms SET screenscraper_id = 2 WHERE slug = 'mastersystem';
UPDATE platforms SET screenscraper_id = 8 WHERE slug = 'dreamcast';
UPDATE platforms SET screenscraper_id = 18 WHERE slug = 'gamecube';
UPDATE platforms SET screenscraper_id = 16 WHERE slug = 'wii';
UPDATE platforms SET screenscraper_id = 13 WHERE slug = 'pce';
UPDATE platforms SET screenscraper_id = 58 WHERE slug = 'ps2';
UPDATE platforms SET screenscraper_id = 59 WHERE slug = 'psp';

-- ScreenScraper response cache
CREATE TABLE IF NOT EXISTS screenscraper_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rom_id INTEGER NOT NULL UNIQUE REFERENCES roms(id) ON DELETE CASCADE,
    screenscraper_game_id INTEGER,
    raw_response TEXT,
    fetched_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_screenscraper_cache_rom_id ON screenscraper_cache(rom_id);
