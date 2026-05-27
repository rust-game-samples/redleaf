use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CategoryWithCount {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub post_count: i64,
}

impl Category {
    pub async fn find_all(pool: &DbPool) -> Result<Vec<Category>, sqlx::Error> {
        sqlx::query_as::<_, Category>(
            "SELECT * FROM categories ORDER BY name ASC",
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_all_with_count(pool: &DbPool) -> Result<Vec<CategoryWithCount>, sqlx::Error> {
        sqlx::query_as::<_, CategoryWithCount>(
            r#"
            SELECT c.*, COUNT(p.id) AS post_count
            FROM categories c
            LEFT JOIN posts p ON p.category_id = c.id
            GROUP BY c.id
            ORDER BY c.name ASC
            "#,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_slug(pool: &DbPool, slug: &str) -> Result<Option<Category>, sqlx::Error> {
        sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE slug = ?")
            .bind(slug)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<Option<Category>, sqlx::Error> {
        sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn create(pool: &DbPool, name: &str, slug: &str) -> Result<Category, sqlx::Error> {
        sqlx::query_as::<_, Category>(
            "INSERT INTO categories (name, slug) VALUES (?, ?) RETURNING *",
        )
        .bind(name)
        .bind(slug)
        .fetch_one(pool)
        .await
    }

    pub async fn update(pool: &DbPool, id: i64, name: &str, slug: &str) -> Result<Category, sqlx::Error> {
        sqlx::query_as::<_, Category>(
            "UPDATE categories SET name = ?, slug = ? WHERE id = ? RETURNING *",
        )
        .bind(name)
        .bind(slug)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM categories WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}