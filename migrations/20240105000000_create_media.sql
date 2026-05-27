CREATE TABLE IF NOT EXISTS media (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    filename      TEXT NOT NULL UNIQUE,
    original_name TEXT NOT NULL,
    mime_type     TEXT NOT NULL,
    size          INTEGER NOT NULL DEFAULT 0,
    url           TEXT NOT NULL,
    uploaded_by   INTEGER REFERENCES users(id) ON DELETE SET NULL,
    created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_media_created ON media (created_at DESC);