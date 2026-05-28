-- ─── ナビゲーションメニュー ──────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS nav_menus (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    location   TEXT NOT NULL DEFAULT '',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_nav_menus_location ON nav_menus (location);

-- ─── メニューアイテム ────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS nav_menu_items (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    menu_id    INTEGER NOT NULL REFERENCES nav_menus(id) ON DELETE CASCADE,
    parent_id  INTEGER REFERENCES nav_menu_items(id) ON DELETE SET NULL,
    item_type  TEXT NOT NULL DEFAULT 'custom'
                    CHECK(item_type IN ('custom','page','category','post')),
    label      TEXT NOT NULL,
    url        TEXT NOT NULL DEFAULT '',
    ref_id     INTEGER,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_nav_menu_items_menu ON nav_menu_items (menu_id, sort_order ASC);

-- デフォルトメニューの挿入
INSERT OR IGNORE INTO nav_menus (name, location) VALUES ('Primary Menu', 'primary');
INSERT OR IGNORE INTO nav_menus (name, location) VALUES ('Footer Menu', 'footer');