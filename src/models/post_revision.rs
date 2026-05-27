use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

use crate::db::DbPool;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PostRevision {
    pub id: i64,
    pub post_id: i64,
    pub title: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
}

impl PostRevision {
    /// Save the current post state as a revision before updating.
    pub async fn save(
        pool: &DbPool,
        post_id: i64,
        title: &str,
        content: &str,
        excerpt: Option<&str>,
        created_by: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO post_revisions (post_id, title, content, excerpt, created_by)
               VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(post_id)
        .bind(title)
        .bind(content)
        .bind(excerpt)
        .bind(created_by)
        .execute(pool)
        .await?;

        // Retain only the 10 most recent revisions
        sqlx::query(
            r#"DELETE FROM post_revisions
               WHERE post_id = ?
                 AND id NOT IN (
                     SELECT id FROM post_revisions
                     WHERE post_id = ?
                     ORDER BY created_at DESC
                     LIMIT 10
                 )"#,
        )
        .bind(post_id)
        .bind(post_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_post(
        pool: &DbPool,
        post_id: i64,
    ) -> Result<Vec<PostRevision>, sqlx::Error> {
        sqlx::query_as::<_, PostRevision>(
            "SELECT * FROM post_revisions WHERE post_id = ? ORDER BY created_at DESC",
        )
        .bind(post_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(
        pool: &DbPool,
        id: i64,
    ) -> Result<Option<PostRevision>, sqlx::Error> {
        sqlx::query_as::<_, PostRevision>("SELECT * FROM post_revisions WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }
}