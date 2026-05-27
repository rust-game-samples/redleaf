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
    models::{Post, PostWithAuthor, Setting, Tag},
    util::{render, Pagination, PER_PAGE},
};

#[derive(Template)]
#[template(path = "posts/list.html")]
struct PostListTemplate {
    posts: Vec<Post>,
    post_url_type: String,
    paging: Pagination,
    site_name: String,
}

#[derive(Template)]
#[template(path = "posts/show.html")]
struct PostShowTemplate {
    post: PostWithAuthor,
    html_content: String,
    tags: Vec<Tag>,
    site_name: String,
}

#[derive(Deserialize, Default)]
struct PageQuery {
    page: Option<i64>,
}

pub fn post_routes() -> Router<DbPool> {
    Router::new()
        .route("/", get(list_posts))
        .route("/{param}", get(show_post))
}

async fn list_posts(
    State(pool): State<DbPool>,
    Query(q): Query<PageQuery>,
) -> Result<Response, AppError> {
    let page = q.page.unwrap_or(1).max(1);

    let (posts, total, post_url_type, site_name) = tokio::join!(
        Post::find_published_paginated(&pool, page, PER_PAGE),
        Post::count_published(&pool),
        Setting::post_url_type(&pool),
        Setting::site_name(&pool),
    );

    let paging = Pagination::new(page, total?, PER_PAGE, "/posts");
    render(PostListTemplate { posts: posts?, post_url_type, paging, site_name })
}

async fn show_post(
    State(pool): State<DbPool>,
    Path(param): Path<String>,
) -> Result<Response, AppError> {
    let (url_type, site_name) = tokio::join!(
        Setting::post_url_type(&pool),
        Setting::site_name(&pool),
    );

    let post = if url_type == "id" {
        let id = param
            .parse::<i64>()
            .map_err(|_| AppError::NotFound)?;
        Post::find_by_id_with_author(&pool, id).await?
    } else {
        Post::find_by_slug_with_author(&pool, &param).await?
    };

    let post = post.ok_or(AppError::NotFound)?;
    let tags = Tag::find_by_post(&pool, post.id).await?;
    let html_content = markdown_to_html(&post.content);
    render(PostShowTemplate { post, html_content, tags, site_name })
}

pub fn markdown_to_html_pub(markdown: &str) -> String {
    markdown_to_html(markdown)
}

fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};

    let normalized = markdown.replace("\r\n", "\n");

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(&normalized, options);
    let mut output = String::new();
    html::push_html(&mut output, parser);
    output
}