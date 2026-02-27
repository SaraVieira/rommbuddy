-- Add MD5 partial index for fast hash lookups during dedup
CREATE INDEX IF NOT EXISTS idx_roms_hash_md5 ON roms(hash_md5) WHERE hash_md5 IS NOT NULL;

-- Drop the unique constraint on (platform_id, file_name) to allow dedup across different filenames.
-- SQLite can't DROP INDEX on a UNIQUE index created with CREATE UNIQUE INDEX, so we recreate it as non-unique.
DROP INDEX IF EXISTS idx_roms_platform_file;
CREATE INDEX IF NOT EXISTS idx_roms_platform_file ON roms(platform_id, file_name);

-- Add per-source file_name and hash_md5 columns to source_roms
-- Each source may know the ROM by a different filename or hash
ALTER TABLE source_roms ADD COLUMN file_name TEXT;
ALTER TABLE source_roms ADD COLUMN hash_md5 TEXT;
