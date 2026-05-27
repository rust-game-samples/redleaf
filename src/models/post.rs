use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub published: bool,
    pub author_id: Option<i64>,
    pub category_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Post with joined author + category names.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PostWithAuthor {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub published: bool,
    pub author_id: Option<i64>,
    pub category_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author_username: Option<String>,
    pub category_name: Option<String>,
    pub category_slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub published: bool,
    pub author_id: Option<i64>,
    pub category_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    /// None = keep existing, Some(None) = clear, Some(Some(s)) = set
    pub excerpt: Option<Option<String>>,
    pub published: Option<bool>,
    /// None = keep existing, Some(None) = clear, Some(Some(id)) = set
    pub category_id: Option<Option<i64>>,
}

// ─── Base query fragment ──────────────────────────────────────────────────────

const WITH_AUTHOR_SELECT: &str = r#"
    SELECT p.*,
        u.username AS author_username,
        c.name     AS category_name,
        c.slug     AS category_slug
    FROM posts p
    LEFT JOIN users      u ON u.id = p.author_id
    LEFT JOIN categories c ON c.id = p.category_id
"#;

impl Post {
    // ─── Public queries ───────────────────────────────────────────────────────

    pub async fn find_all(pool: &DbPool) -> Result<Vec<Post>, sqlx::Error> {
        sqlx::query_as::<_, Post>(
            "SELECT * FROM posts WHERE published = 1 ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_published_paginated(
        pool: &DbPool,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<Post>, sqlx::Error> {
        let offset = (page - 1) * per_page;
        sqlx::query_as::<_, Post>(
            "SELECT * FROM posts WHERE published = 1 ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    pub async fn count_published(pool: &DbPool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COUNT(*) FROM posts WHERE published = 1")
            .fetch_one(pool)
            .await
    }

    // ─── Admin queries ────────────────────────────────────────────────────────

    pub async fn find_all_admin(pool: &DbPool) -> Result<Vec<Post>, sqlx::Error> {
        sqlx::query_as::<_, Post>("SELECT * FROM posts ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
    }

    pub async fn find_all_admin_with_author(
        pool: &DbPool,
    ) -> Result<Vec<PostWithAuthor>, sqlx::Error> {
        let sql = format!("{} ORDER BY p.created_at DESC", WITH_AUTHOR_SELECT);
        sqlx::query_as::<_, PostWithAuthor>(&sql)
            .fetch_all(pool)
            .await
    }

    pub async fn find_admin_paginated(
        pool: &DbPool,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<PostWithAuthor>, sqlx::Error> {
        let offset = (page - 1) * per_page;
        let sql = format!("{} ORDER BY p.created_at DESC LIMIT ? OFFSET ?", WITH_AUTHOR_SELECT);
        sqlx::query_as::<_, PostWithAuthor>(&sql)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await
    }

    pub async fn count_all(pool: &DbPool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COUNT(*) FROM posts")
            .fetch_one(pool)
            .await
    }

    // ─── By ID / slug ─────────────────────────────────────────────────────────

    pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<Option<Post>, sqlx::Error> {
        sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_slug(pool: &DbPool, slug: &str) -> Result<Option<Post>, sqlx::Error> {
        sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE slug = ?")
            .bind(slug)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_id_with_author(
        pool: &DbPool,
        id: i64,
    ) -> Result<Option<PostWithAuthor>, sqlx::Error> {
        let sql = format!("{} WHERE p.id = ?", WITH_AUTHOR_SELECT);
        sqlx::query_as::<_, PostWithAuthor>(&sql)
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_slug_with_author(
        pool: &DbPool,
        slug: &str,
    ) -> Result<Option<PostWithAuthor>, sqlx::Error> {
        let sql = format!("{} WHERE p.slug = ?", WITH_AUTHOR_SELECT);
        sqlx::query_as::<_, PostWithAuthor>(&sql)
            .bind(slug)
            .fetch_optional(pool)
            .await
    }

    // ─── Taxonomy queries ─────────────────────────────────────────────────────

    pub async fn find_published_paginated_with_author(
        pool: &DbPool,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<PostWithAuthor>, sqlx::Error> {
        let offset = (page - 1) * per_page;
        let sql = format!(
            "{} WHERE p.published = 1 ORDER BY p.created_at DESC LIMIT ? OFFSET ?",
            WITH_AUTHOR_SELECT
        );
        sqlx::query_as::<_, PostWithAuthor>(&sql)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await
    }

    pub async fn find_by_category(
        pool: &DbPool,
        category_slug: &str,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<PostWithAuthor>, sqlx::Error> {
        let offset = (page - 1) * per_page;
        sqlx::query_as::<_, PostWithAuthor>(
            r#"
            SELECT p.*,
                u.username AS author_username,
                c.name     AS category_name,
                c.slug     AS category_slug
            FROM posts p
            LEFT JOIN users      u ON u.id = p.author_id
            JOIN      categories c ON c.id = p.category_id
            WHERE p.published = 1 AND c.slug = ?
            ORDER BY p.created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(category_slug)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    pub async fn count_by_category(
        pool: &DbPool,
        category_slug: &str,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM posts p
            JOIN categories c ON c.id = p.category_id
            WHERE p.published = 1 AND c.slug = ?
            "#,
        )
        .bind(category_slug)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_tag(
        pool: &DbPool,
        tag_slug: &str,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<PostWithAuthor>, sqlx::Error> {
        let offset = (page - 1) * per_page;
        sqlx::query_as::<_, PostWithAuthor>(
            r#"
            SELECT p.*,
                u.username AS author_username,
                c.name     AS category_name,
                c.slug     AS category_slug
            FROM posts p
            LEFT JOIN users      u  ON u.id  = p.author_id
            LEFT JOIN categories c  ON c.id  = p.category_id
            JOIN      post_tags  pt ON pt.post_id = p.id
            JOIN      tags       t  ON t.id  = pt.tag_id
            WHERE p.published = 1 AND t.slug = ?
            ORDER BY p.created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(tag_slug)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    pub async fn count_by_tag(pool: &DbPool, tag_slug: &str) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM posts p
            JOIN post_tags pt ON pt.post_id = p.id
            JOIN tags      t  ON t.id = pt.tag_id
            WHERE p.published = 1 AND t.slug = ?
            "#,
        )
        .bind(tag_slug)
        .fetch_one(pool)
        .await
    }

    // ─── Tags ─────────────────────────────────────────────────────────────────

    pub async fn set_tags(
        pool: &DbPool,
        post_id: i64,
        tag_ids: &[i64],
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM post_tags WHERE post_id = ?")
            .bind(post_id)
            .execute(pool)
            .await?;
        for &tag_id in tag_ids {
            sqlx::query(
                "INSERT OR IGNORE INTO post_tags (post_id, tag_id) VALUES (?, ?)",
            )
            .bind(post_id)
            .bind(tag_id)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    // ─── Write operations ─────────────────────────────────────────────────────

    pub async fn create(pool: &DbPool, p: CreatePost) -> Result<Post, sqlx::Error> {
        let now = Utc::now();
        sqlx::query_as::<_, Post>(
            r#"
            INSERT INTO posts (title, slug, content, excerpt, published, author_id, category_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&p.title)
        .bind(&p.slug)
        .bind(&p.content)
        .bind(&p.excerpt)
        .bind(p.published)
        .bind(p.author_id)
        .bind(p.category_id)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &DbPool,
        id: i64,
        u: UpdatePost,
    ) -> Result<Post, sqlx::Error> {
        let now = Utc::now();
        let cur = Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        sqlx::query_as::<_, Post>(
            r#"
            UPDATE posts
            SET title = ?, slug = ?, content = ?, excerpt = ?,
                published = ?, category_id = ?, updated_at = ?
            WHERE id = ?
            RETURNING *
            "#,
        )
        .bind(u.title.unwrap_or(cur.title))
        .bind(u.slug.unwrap_or(cur.slug))
        .bind(u.content.unwrap_or(cur.content))
        .bind(match u.excerpt {
            None => cur.excerpt,
            Some(v) => v,
        })
        .bind(u.published.unwrap_or(cur.published))
        .bind(match u.category_id {
            None => cur.category_id,
            Some(v) => v,
        })
        .bind(now)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM posts WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    // ─── Full-text search ─────────────────────────────────────────────────────

    pub async fn search(pool: &DbPool, fts_query: &str) -> Result<Vec<Post>, sqlx::Error> {
        sqlx::query_as::<_, Post>(
            r#"
            SELECT p.* FROM posts p
            JOIN posts_fts ON posts_fts.rowid = p.id
            WHERE posts_fts MATCH ? AND p.published = 1
            ORDER BY rank
            LIMIT 30
            "#,
        )
        .bind(fts_query)
        .fetch_all(pool)
        .await
    }
}