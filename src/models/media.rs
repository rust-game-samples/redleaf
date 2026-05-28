use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MediaVariant {
    pub id: i64,
    pub media_id: i64,
    pub size_name: String,
    pub filename: String,
    pub url: String,
    pub width: i64,
    pub height: i64,
    pub file_size: i64,
}

impl MediaVariant {
    pub fn is_webp(&self) -> bool {
        self.size_name.ends_with("-webp")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Media {
    pub id: i64,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: i64,
    pub url: String,
    pub uploaded_by: Option<i64>,
    pub created_at: DateTime<Utc>,
}

impl Media {
    pub async fn find_all(pool: &DbPool) -> Result<Vec<Media>, sqlx::Error> {
        sqlx::query_as::<_, Media>(
            "SELECT * FROM media ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<Option<Media>, sqlx::Error> {
        sqlx::query_as::<_, Media>("SELECT * FROM media WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn create(
        pool: &DbPool,
        filename: &str,
        original_name: &str,
        mime_type: &str,
        size: i64,
        url: &str,
        uploaded_by: Option<i64>,
    ) -> Result<Media, sqlx::Error> {
        sqlx::query_as::<_, Media>(
            "INSERT INTO media (filename, original_name, mime_type, size, url, uploaded_by)
             VALUES (?, ?, ?, ?, ?, ?) RETURNING *",
        )
        .bind(filename)
        .bind(original_name)
        .bind(mime_type)
        .bind(size)
        .bind(url)
        .bind(uploaded_by)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &DbPool, id: i64) -> Result<Option<Media>, sqlx::Error> {
        sqlx::query_as::<_, Media>("DELETE FROM media WHERE id = ? RETURNING *")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn create_variant(
        pool: &DbPool,
        media_id: i64,
        size_name: &str,
        filename: &str,
        url: &str,
        width: u32,
        height: u32,
        file_size: usize,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO media_variants (media_id, size_name, filename, url, width, height, file_size)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(media_id)
        .bind(size_name)
        .bind(filename)
        .bind(url)
        .bind(width as i64)
        .bind(height as i64)
        .bind(file_size as i64)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn find_variants(pool: &DbPool, media_id: i64) -> Result<Vec<MediaVariant>, sqlx::Error> {
        sqlx::query_as::<_, MediaVariant>(
            "SELECT * FROM media_variants WHERE media_id = ? ORDER BY width ASC",
        )
        .bind(media_id)
        .fetch_all(pool)
        .await
    }

    pub async fn count_variants(pool: &DbPool, media_id: i64) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COUNT(*) FROM media_variants WHERE media_id = ?")
            .bind(media_id)
            .fetch_one(pool)
            .await
    }

    pub fn is_image(&self) -> bool {
        self.mime_type.starts_with("image/")
    }

    pub fn human_size(&self) -> String {
        let bytes = self.size as f64;
        if bytes < 1024.0 {
            format!("{} B", self.size)
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.1} KB", bytes / 1024.0)
        } else {
            format!("{:.1} MB", bytes / (1024.0 * 1024.0))
        }
    }
}