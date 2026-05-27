use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::Claims,
    db::DbPool,
    errors::{ApiError, AppError},
    models::{
        Post, PostWithAuthor,
        post::{CreatePost, UpdatePost},
    },
    util::{slugify, PER_PAGE},
};

// ─── Response types ───────────────────────────────────────────────────────────

#[derive(Serialize)]
struct PostListResponse {
    items: Vec<PostWithAuthor>,
    total: i64,
    page: i64,
    per_page: i64,
    total_pages: i64,
}

// ─── Request types ────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
struct PageQuery {
    page: Option<i64>,
}

#[derive(Deserialize)]
pub struct ApiCreatePost {
    pub title: String,
    pub slug: Option<String>,
    pub content: String,
    pub excerpt: Option<String>,
    pub published: Option<bool>,
}

#[derive(Deserialize)]
pub struct ApiUpdatePost {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    /// `null` or `""` clears the excerpt; omit to keep existing.
    pub excerpt: Option<String>,
    pub published: Option<bool>,
}

// ─── Routers ──────────────────────────────────────────────────────────────────

/// Public (no auth): read-only post endpoints.
pub fn api_public_routes() -> Router<DbPool> {
    Router::new()
        .route("/posts", get(list_posts))
        .route("/posts/{id}", get(show_post))
}

/// Protected (Bearer JWT required): write endpoints.
pub fn api_protected_routes() -> Router<DbPool> {
    Router::new()
        .route("/posts", post(create_post))
        .route("/posts/{id}", put(update_post).delete(delete_post))
}

// ─── Public handlers ──────────────────────────────────────────────────────────

async fn list_posts(
    State(pool): State<DbPool>,
    Query(q): Query<PageQuery>,
) -> Result<Response, ApiError> {
    let page = q.page.unwrap_or(1).max(1);

    let (posts, total) = tokio::join!(
        Post::find_published_paginated_with_author(&pool, page, PER_PAGE),
        Post::count_published(&pool),
    );

    let total = total?;
    let total_pages = ((total as f64) / (PER_PAGE as f64)).ceil() as i64;
    let total_pages = total_pages.max(1);

    Ok(Json(PostListResponse {
        items: posts?,
        total,
        page,
        per_page: PER_PAGE,
        total_pages,
    })
    .into_response())
}

async fn show_post(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let post = Post::find_by_id_with_author(&pool, id)
        .await?
        .ok_or(AppError::NotFound)?;

    if !post.published {
        return Err(AppError::NotFound.into());
    }

    Ok(Json(post).into_response())
}

// ─── Protected handlers ───────────────────────────────────────────────────────

async fn create_post(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<ApiCreatePost>,
) -> Result<Response, ApiError> {
    let slug = match body.slug.as_deref() {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => slugify(&body.title),
    };

    let payload = CreatePost {
        title: body.title,
        slug,
        content: body.content,
        excerpt: body.excerpt.filter(|s| !s.trim().is_empty()),
        published: body.published.unwrap_or(false),
        sticky: false,
        author_id: Some(claims.sub),
        category_id: None,
        featured_image_id: None,
        scheduled_at: None,
    };

    let post = Post::create(&pool, payload).await.map_err(|e| {
        AppError::from_db_create(e, "Slug already exists")
    })?;

    Ok((StatusCode::CREATED, Json(post)).into_response())
}

async fn update_post(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Json(body): Json<ApiUpdatePost>,
) -> Result<Response, ApiError> {
    let payload = UpdatePost {
        title: body.title,
        slug: body.slug,
        content: body.content,
        excerpt: body.excerpt.map(|s| {
            if s.trim().is_empty() { None } else { Some(s) }
        }),
        published: body.published,
        sticky: None,
        category_id: None,
        featured_image_id: None,
        scheduled_at: None,
    };

    let post = Post::update(&pool, id, payload).await.map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound,
        other => AppError::Database(other),
    })?;

    Ok(Json(post).into_response())
}

async fn delete_post(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    Post::find_by_id(&pool, id)
        .await?
        .ok_or(AppError::NotFound)?;

    Post::delete(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}