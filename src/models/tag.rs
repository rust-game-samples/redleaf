use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{db::DbPool, util::slugify};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TagWithCount {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub post_count: i64,
}

impl Tag {
    pub async fn find_all_with_count(pool: &DbPool) -> Result<Vec<TagWithCount>, sqlx::Error> {
        sqlx::query_as::<_, TagWithCount>(
            r#"
            SELECT t.*, COUNT(pt.post_id) AS post_count
            FROM tags t
            LEFT JOIN post_tags pt ON pt.tag_id = t.id
            GROUP BY t.id
            ORDER BY t.name ASC
            "#,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_slug(pool: &DbPool, slug: &str) -> Result<Option<Tag>, sqlx::Error> {
        sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE slug = ?")
            .bind(slug)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_post(pool: &DbPool, post_id: i64) -> Result<Vec<Tag>, sqlx::Error> {
        sqlx::query_as::<_, Tag>(
            r#"
            SELECT t.*
            FROM tags t
            JOIN post_tags pt ON pt.tag_id = t.id
            WHERE pt.post_id = ?
            ORDER BY t.name ASC
            "#,
        )
        .bind(post_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_or_create(pool: &DbPool, name: &str) -> Result<Tag, sqlx::Error> {
        let slug = slugify(name);
        if let Some(tag) = sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE slug = ?")
            .bind(&slug)
            .fetch_optional(pool)
            .await?
        {
            return Ok(tag);
        }
        sqlx::query_as::<_, Tag>(
            "INSERT INTO tags (name, slug) VALUES (?, ?) RETURNING *",
        )
        .bind(name)
        .bind(slug)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM tags WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}