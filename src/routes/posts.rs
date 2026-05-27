use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use crate::{db::DbPool, models::{Post, PostWithAuthor, Setting}, util::render};

#[derive(Template)]
#[template(path = "posts/list.html")]
struct PostListTemplate {
    posts: Vec<Post>,
    post_url_type: String,
}

#[derive(Template)]
#[template(path = "posts/show.html")]
struct PostShowTemplate {
    post: PostWithAuthor,
    html_content: String,
}

pub fn post_routes() -> Router<DbPool> {
    Router::new()
        .route("/", get(list_posts))
        .route("/{param}", get(show_post))
}

async fn list_posts(State(pool): State<DbPool>) -> Response {
    let (posts, post_url_type) = tokio::join!(
        Post::find_all(&pool),
        Setting::post_url_type(&pool),
    );
    match posts {
        Ok(posts) => render(PostListTemplate { posts, post_url_type }),
        Err(e) => {
            tracing::error!("Failed to fetch posts: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error loading posts").into_response()
        }
    }
}

async fn show_post(State(pool): State<DbPool>, Path(param): Path<String>) -> Response {
    let url_type = Setting::post_url_type(&pool).await;

    let result = if url_type == "id" {
        match param.parse::<i64>() {
            Ok(id) => Post::find_by_id_with_author(&pool, id).await,
            Err(_) => return (StatusCode::NOT_FOUND, "Post not found").into_response(),
        }
    } else {
        Post::find_by_slug_with_author(&pool, &param).await
    };

    match result {
        Ok(Some(post)) => {
            let html_content = markdown_to_html(&post.content);
            render(PostShowTemplate { post, html_content })
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Post not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch post {}: {}", param, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error loading post").into_response()
        }
    }
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