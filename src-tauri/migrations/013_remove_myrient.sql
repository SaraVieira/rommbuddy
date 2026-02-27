-- Remove Myrient source type support.
-- SQLite doesn't support ALTER TABLE to modify CHECK constraints,
-- so we recreate the sources table without the myrient option.

-- First delete any existing myrient sources and their associated data
DELETE FROM source_roms WHERE source_id IN (SELECT id FROM sources WHERE source_type = 'myrient');
DELETE FROM library WHERE source_id IN (SELECT id FROM sources WHERE source_type = 'myrient');
DELETE FROM downloads WHERE source_id IN (SELECT id FROM sources WHERE source_type = 'myrient');
DELETE FROM sources WHERE source_type = 'myrient';

-- Recreate sources table without myrient in the CHECK constraint
CREATE TABLE sources_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source_type TEXT NOT NULL CHECK (source_type IN ('local', 'romm')),
    url TEXT,
    credentials TEXT NOT NULL DEFAULT '{}',
    settings TEXT NOT NULL DEFAULT '{}',
    enabled INTEGER NOT NULL DEFAULT 1,
    last_synced_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

INSERT INTO sources_new SELECT * FROM sources;
DROP TABLE sources;
ALTER TABLE sources_new RENAME TO sources;
