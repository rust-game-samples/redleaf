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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub published: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub excerpt: Option<String>,
    pub published: Option<bool>,
}

impl Post {
    // Find all posts
    pub async fn find_all(pool: &DbPool) -> Result<Vec<Post>, sqlx::Error> {
        sqlx::query_as::<_, Post>(
            "SELECT * FROM posts WHERE published = 1 ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await
    }

    // Find post by ID
    pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<Option<Post>, sqlx::Error> {
        sqlx::query_as::<_, Post>(
            "SELECT * FROM posts WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    // Find post by slug
    pub async fn find_by_slug(pool: &DbPool, slug: &str) -> Result<Option<Post>, sqlx::Error> {
        sqlx::query_as::<_, Post>(
            "SELECT * FROM posts WHERE slug = ?"
        )
        .bind(slug)
        .fetch_optional(pool)
        .await
    }

    // Create new post
    pub async fn create(pool: &DbPool, create_post: CreatePost) -> Result<Post, sqlx::Error> {
        let now = Utc::now();

        sqlx::query_as::<_, Post>(
            r#"
            INSERT INTO posts (title, slug, content, excerpt, published, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#
        )
        .bind(&create_post.title)
        .bind(&create_post.slug)
        .bind(&create_post.content)
        .bind(&create_post.excerpt)
        .bind(create_post.published)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    // Update post
    pub async fn update(
        pool: &DbPool,
        id: i64,
        update_post: UpdatePost,
    ) -> Result<Post, sqlx::Error> {
        let now = Utc::now();

        // First get the current post
        let current_post = Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        sqlx::query_as::<_, Post>(
            r#"
            UPDATE posts
            SET title = ?,
                slug = ?,
                content = ?,
                excerpt = ?,
                published = ?,
                updated_at = ?
            WHERE id = ?
            RETURNING *
            "#
        )
        .bind(update_post.title.unwrap_or(current_post.title))
        .bind(update_post.slug.unwrap_or(current_post.slug))
        .bind(update_post.content.unwrap_or(current_post.content))
        .bind(update_post.excerpt.or(current_post.excerpt))
        .bind(update_post.published.unwrap_or(current_post.published))
        .bind(now)
        .bind(id)
        .fetch_one(pool)
        .await
    }

    // Delete post
    pub async fn delete(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM posts WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
