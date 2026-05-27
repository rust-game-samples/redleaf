use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use crate::{db::DbPool, models::Post, util::render};

#[derive(Template)]
#[template(path = "posts/list.html")]
struct PostListTemplate {
    posts: Vec<Post>,
}

#[derive(Template)]
#[template(path = "posts/show.html")]
struct PostShowTemplate {
    post: Post,
    html_content: String,
}

pub fn post_routes() -> Router<DbPool> {
    Router::new()
        .route("/", get(list_posts))
        .route("/{id}", get(show_post))
}

async fn list_posts(State(pool): State<DbPool>) -> Response {
    match Post::find_all(&pool).await {
        Ok(posts) => render(PostListTemplate { posts }),
        Err(e) => {
            tracing::error!("Failed to fetch posts: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error loading posts").into_response()
        }
    }
}

async fn show_post(State(pool): State<DbPool>, Path(id): Path<i64>) -> Response {
    match Post::find_by_id(&pool, id).await {
        Ok(Some(post)) => {
            let html_content = markdown_to_html(&post.content);
            render(PostShowTemplate { post, html_content })
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Post not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch post {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error loading post").into_response()
        }
    }
}

fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(markdown, options);
    let mut output = String::new();
    html::push_html(&mut output, parser);
    output
}