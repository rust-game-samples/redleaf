use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PostMeta {
    pub id: i64,
    pub post_id: i64,
    pub meta_key: String,
    pub meta_value: String,
}

impl PostMeta {
    pub async fn get(pool: &DbPool, post_id: i64, key: &str) -> Option<String> {
        sqlx::query_scalar(
            "SELECT meta_value FROM post_meta WHERE post_id = ? AND meta_key = ?",
        )
        .bind(post_id)
        .bind(key)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
    }

    pub async fn get_all(pool: &DbPool, post_id: i64) -> Result<Vec<PostMeta>, sqlx::Error> {
        sqlx::query_as::<_, PostMeta>(
            "SELECT * FROM post_meta WHERE post_id = ? ORDER BY meta_key ASC",
        )
        .bind(post_id)
        .fetch_all(pool)
        .await
    }

    pub async fn set(
        pool: &DbPool,
        post_id: i64,
        key: &str,
        value: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO post_meta (post_id, meta_key, meta_value) VALUES (?, ?, ?)
               ON CONFLICT(post_id, meta_key) DO UPDATE SET meta_value = excluded.meta_value"#,
        )
        .bind(post_id)
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete(pool: &DbPool, post_id: i64, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM post_meta WHERE post_id = ? AND meta_key = ?")
            .bind(post_id)
            .bind(key)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Replace all meta for a post (used on form save).
    pub async fn replace_all(
        pool: &DbPool,
        post_id: i64,
        pairs: &[(String, String)],
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM post_meta WHERE post_id = ?")
            .bind(post_id)
            .execute(pool)
            .await?;
        for (k, v) in pairs {
            let k = k.trim();
            if k.is_empty() { continue; }
            Self::set(pool, post_id, k, v.trim()).await?;
        }
        Ok(())
    }
}