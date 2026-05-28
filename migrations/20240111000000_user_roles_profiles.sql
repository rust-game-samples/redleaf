-- ─── users テーブル拡張 ──────────────────────────────────────────────────────
-- ロール: administrator / editor / author / contributor / subscriber
ALTER TABLE users ADD COLUMN role         TEXT NOT NULL DEFAULT 'administrator';
ALTER TABLE users ADD COLUMN display_name TEXT NOT NULL DEFAULT '';
ALTER TABLE users ADD COLUMN bio          TEXT NOT NULL DEFAULT '';
ALTER TABLE users ADD COLUMN website      TEXT NOT NULL DEFAULT '';
ALTER TABLE users ADD COLUMN avatar_url   TEXT NOT NULL DEFAULT '';