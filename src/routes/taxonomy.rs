use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::{
    db::DbPool,
    errors::AppError,
    models::{Category, Post, PostWithAuthor, Tag},
    util::{render, Pagination, PER_PAGE},
};

#[derive(Template)]
#[template(path = "taxonomy/category.html")]
struct CategoryPageTemplate {
    category: Category,
    posts: Vec<PostWithAuthor>,
    paging: Pagination,
}

#[derive(Template)]
#[template(path = "taxonomy/tag.html")]
struct TagPageTemplate {
    tag: Tag,
    posts: Vec<PostWithAuthor>,
    paging: Pagination,
}

#[derive(Deserialize, Default)]
struct PageQuery {
    page: Option<i64>,
}

pub fn taxonomy_routes() -> Router<DbPool> {
    Router::new()
        .route("/categories/{slug}", get(category_page))
        .route("/tags/{slug}", get(tag_page))
}

async fn category_page(
    State(pool): State<DbPool>,
    Path(slug): Path<String>,
    Query(q): Query<PageQuery>,
) -> Result<Response, AppError> {
    let category = Category::find_by_slug(&pool, &slug)
        .await?
        .ok_or(AppError::NotFound)?;
    let page = q.page.unwrap_or(1).max(1);
    let (posts, total) = tokio::join!(
        Post::find_by_category(&pool, &slug, page, PER_PAGE),
        Post::count_by_category(&pool, &slug),
    );
    let base = format!("/categories/{}", slug);
    let paging = Pagination::new(page, total?, PER_PAGE, &base);
    render(CategoryPageTemplate { category, posts: posts?, paging })
}

async fn tag_page(
    State(pool): State<DbPool>,
    Path(slug): Path<String>,
    Query(q): Query<PageQuery>,
) -> Result<Response, AppError> {
    let tag = Tag::find_by_slug(&pool, &slug)
        .await?
        .ok_or(AppError::NotFound)?;
    let page = q.page.unwrap_or(1).max(1);
    let (posts, total) = tokio::join!(
        Post::find_by_tag(&pool, &slug, page, PER_PAGE),
        Post::count_by_tag(&pool, &slug),
    );
    let base = format!("/tags/{}", slug);
    let paging = Pagination::new(page, total?, PER_PAGE, &base);
    render(TagPageTemplate { tag, posts: posts?, paging })
}