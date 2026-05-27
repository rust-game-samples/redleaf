use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Page {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub template: String,
    pub parent_id: Option<i64>,
    pub status: String,
    pub sort_order: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Page {
    pub fn is_published(&self) -> bool {
        self.status == "published"
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePage {
    pub title: String,
    pub slug: String,
    pub content: String,
    pub status: String,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePage {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub parent_id: Option<Option<i64>>,
}

impl Page {
    pub async fn find_all(pool: &DbPool) -> Result<Vec<Page>, sqlx::Error> {
        sqlx::query_as::<_, Page>(
            "SELECT * FROM pages ORDER BY sort_order ASC, created_at ASC",
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_published(pool: &DbPool) -> Result<Vec<Page>, sqlx::Error> {
        sqlx::query_as::<_, Page>(
            "SELECT * FROM pages WHERE status = 'published' ORDER BY sort_order ASC",
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<Option<Page>, sqlx::Error> {
        sqlx::query_as::<_, Page>("SELECT * FROM pages WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_slug(pool: &DbPool, slug: &str) -> Result<Option<Page>, sqlx::Error> {
        sqlx::query_as::<_, Page>(
            "SELECT * FROM pages WHERE slug = ? AND status = 'published'",
        )
        .bind(slug)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &DbPool, p: CreatePage) -> Result<Page, sqlx::Error> {
        let now = Utc::now();
        sqlx::query_as::<_, Page>(
            r#"
            INSERT INTO pages (title, slug, content, status, parent_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&p.title)
        .bind(&p.slug)
        .bind(&p.content)
        .bind(&p.status)
        .bind(p.parent_id)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    pub async fn update(pool: &DbPool, id: i64, u: UpdatePage) -> Result<Page, sqlx::Error> {
        let now = Utc::now();
        let cur = Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;
        sqlx::query_as::<_, Page>(
            r#"
            UPDATE pages
            SET title = ?, slug = ?, content = ?, status = ?, parent_id = ?, updated_at = ?
            WHERE id = ?
            RETURNING *
            "#,
        )
        .bind(u.title.unwrap_or(cur.title))
        .bind(u.slug.unwrap_or(cur.slug))
        .bind(u.content.unwrap_or(cur.content))
        .bind(u.status.unwrap_or(cur.status))
        .bind(match u.parent_id { None => cur.parent_id, Some(v) => v })
        .bind(now)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM pages WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn count(pool: &DbPool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COUNT(*) FROM pages")
            .fetch_one(pool)
            .await
    }
}