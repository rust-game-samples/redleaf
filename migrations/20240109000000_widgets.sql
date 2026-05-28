-- ─── ウィジェットエリア ──────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS widget_areas (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    slug       TEXT NOT NULL UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ─── ウィジェット ────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS widgets (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    area_id     INTEGER NOT NULL REFERENCES widget_areas(id) ON DELETE CASCADE,
    widget_type TEXT NOT NULL CHECK(widget_type IN ('recent_posts','categories','tag_cloud','text','search')),
    title       TEXT NOT NULL DEFAULT '',
    settings    TEXT NOT NULL DEFAULT '{}',
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_widgets_area ON widgets (area_id, sort_order ASC);

-- デフォルトエリアの挿入
INSERT OR IGNORE INTO widget_areas (name, slug) VALUES ('Primary Sidebar', 'sidebar');
INSERT OR IGNORE INTO widget_areas (name, slug) VALUES ('Footer', 'footer');