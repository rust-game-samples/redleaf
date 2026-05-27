-- ─── posts 拡張 ──────────────────────────────────────────────────────────────
ALTER TABLE posts ADD COLUMN sticky           INTEGER NOT NULL DEFAULT 0;
ALTER TABLE posts ADD COLUMN featured_image_id INTEGER REFERENCES media(id) ON DELETE SET NULL;
ALTER TABLE posts ADD COLUMN scheduled_at      DATETIME;

-- ─── 固定ページ ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS pages (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT NOT NULL,
    slug       TEXT NOT NULL UNIQUE,
    content    TEXT NOT NULL DEFAULT '',
    template   TEXT NOT NULL DEFAULT 'default',
    parent_id  INTEGER REFERENCES pages(id) ON DELETE SET NULL,
    status     TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('published','draft')),
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_pages_slug   ON pages (slug);
CREATE INDEX IF NOT EXISTS idx_pages_status ON pages (status);

-- ─── カスタムフィールド ───────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS post_meta (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id    INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    meta_key   TEXT NOT NULL,
    meta_value TEXT NOT NULL DEFAULT '',
    UNIQUE(post_id, meta_key)
);
CREATE INDEX IF NOT EXISTS idx_post_meta_post ON post_meta (post_id);

-- ─── 投稿リビジョン ───────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS post_revisions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id    INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    title      TEXT NOT NULL,
    content    TEXT NOT NULL,
    excerpt    TEXT,
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_post_revisions_post ON post_revisions (post_id, created_at DESC);

-- ─── サイト設定: フロントページ ──────────────────────────────────────────────
INSERT OR IGNORE INTO settings (key, value) VALUES ('front_page_id', '');