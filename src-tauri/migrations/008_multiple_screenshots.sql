-- Allow multiple screenshots per ROM by replacing the unique (rom_id, art_type)
-- index with a unique (rom_id, art_type, url) index to prevent exact duplicates.
DROP INDEX IF EXISTS idx_artwork_rom_type;
CREATE UNIQUE INDEX IF NOT EXISTS idx_artwork_rom_type_url ON artwork(rom_id, art_type, url);
