CREATE TABLE IF NOT EXISTS media_variants (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    media_id  INTEGER NOT NULL REFERENCES media(id) ON DELETE CASCADE,
    size_name TEXT    NOT NULL,
    filename  TEXT    NOT NULL,
    url       TEXT    NOT NULL,
    width     INTEGER NOT NULL DEFAULT 0,
    height    INTEGER NOT NULL DEFAULT 0,
    file_size INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_media_variants_media_id ON media_variants(media_id);