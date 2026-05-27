-- Full-text search index for published posts
CREATE VIRTUAL TABLE IF NOT EXISTS posts_fts USING fts5(
    title,
    body
);

-- Populate from existing published posts
INSERT OR IGNORE INTO posts_fts(rowid, title, body)
SELECT id, title, content FROM posts WHERE published = 1;

-- Insert trigger: only index published posts
CREATE TRIGGER IF NOT EXISTS posts_fts_ai AFTER INSERT ON posts
WHEN new.published = 1
BEGIN
    INSERT INTO posts_fts(rowid, title, body) VALUES (new.id, new.title, new.content);
END;

-- Delete trigger: remove from index
CREATE TRIGGER IF NOT EXISTS posts_fts_ad AFTER DELETE ON posts BEGIN
    DELETE FROM posts_fts WHERE rowid = old.id;
END;

-- Update trigger: re-index (remove unpublished posts from index)
CREATE TRIGGER IF NOT EXISTS posts_fts_au AFTER UPDATE ON posts BEGIN
    DELETE FROM posts_fts WHERE rowid = old.id;
    INSERT INTO posts_fts(rowid, title, body)
    SELECT new.id, new.title, new.content WHERE new.published = 1;
END;