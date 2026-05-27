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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Post with author username resolved via LEFT JOIN.
#[derive(Debug, Clone, FromRow)]
pub struct PostWithAuthor {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub published: bool,
    pub author_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author_username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub published: bool,
    pub author_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    /// None = keep existing, Some(None) = clear, Some(Some(s)) = set
    pub excerpt: Option<Option<String>>,
    pub published: Option<bool>,
}

impl Post {
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

    pub async fn find_all_admin(pool: &DbPool) -> Result<Vec<Post>, sqlx::Error> {
        sqlx::query_as::<_, Post>("SELECT * FROM posts ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
    }

    pub async fn find_all_admin_with_author(
        pool: &DbPool,
    ) -> Result<Vec<PostWithAuthor>, sqlx::Error> {
        sqlx::query_as::<_, PostWithAuthor>(
            r#"
            SELECT p.*, u.username AS author_username
            FROM posts p
            LEFT JOIN users u ON u.id = p.author_id
            ORDER BY p.created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_admin_paginated(
        pool: &DbPool,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<PostWithAuthor>, sqlx::Error> {
        let offset = (page - 1) * per_page;
        sqlx::query_as::<_, PostWithAuthor>(
            r#"
            SELECT p.*, u.username AS author_username
            FROM posts p
            LEFT JOIN users u ON u.id = p.author_id
            ORDER BY p.created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
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
        sqlx::query_as::<_, PostWithAuthor>(
            r#"
            SELECT p.*, u.username AS author_username
            FROM posts p
            LEFT JOIN users u ON u.id = p.author_id
            WHERE p.id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_slug_with_author(
        pool: &DbPool,
        slug: &str,
    ) -> Result<Option<PostWithAuthor>, sqlx::Error> {
        sqlx::query_as::<_, PostWithAuthor>(
            r#"
            SELECT p.*, u.username AS author_username
            FROM posts p
            LEFT JOIN users u ON u.id = p.author_id
            WHERE p.slug = ?
            "#,
        )
        .bind(slug)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &DbPool, create_post: CreatePost) -> Result<Post, sqlx::Error> {
        let now = Utc::now();

        sqlx::query_as::<_, Post>(
            r#"
            INSERT INTO posts (title, slug, content, excerpt, published, author_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&create_post.title)
        .bind(&create_post.slug)
        .bind(&create_post.content)
        .bind(&create_post.excerpt)
        .bind(create_post.published)
        .bind(create_post.author_id)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &DbPool,
        id: i64,
        update_post: UpdatePost,
    ) -> Result<Post, sqlx::Error> {
        let now = Utc::now();

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
            "#,
        )
        .bind(update_post.title.unwrap_or(current_post.title))
        .bind(update_post.slug.unwrap_or(current_post.slug))
        .bind(update_post.content.unwrap_or(current_post.content))
        .bind(match update_post.excerpt {
            None => current_post.excerpt,
            Some(v) => v,
        })
        .bind(update_post.published.unwrap_or(current_post.published))
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
}