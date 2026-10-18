CREATE TABLE subscriptions (
 id TEXT PRIMARY KEY,
 platform TEXT NOT NULL CHECK(platform = 'rss'),
 url TEXT NOT NULL UNIQUE,
 title TEXT NOT NULL,
 last_synced_at TEXT,
 last_error TEXT
);
CREATE TABLE items (
 id TEXT PRIMARY KEY,
 source_id TEXT NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
 external_id TEXT NOT NULL,
 title TEXT NOT NULL,
 url TEXT,
 published_at TEXT,
 fetched_at TEXT NOT NULL,
 UNIQUE(source_id, external_id)
);
CREATE INDEX items_chronological ON items(published_at DESC, id DESC);
CREATE TABLE item_states (
 item_id TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
 is_read INTEGER NOT NULL DEFAULT 0 CHECK(is_read IN (0, 1))
);
-- Metadata only; credentials must never be placed in this database.
CREATE TABLE connected_accounts (
 id TEXT PRIMARY KEY,
 platform TEXT NOT NULL,
 display_name TEXT NOT NULL,
 credential_ref TEXT,
 status TEXT NOT NULL DEFAULT 'disconnected'
);
