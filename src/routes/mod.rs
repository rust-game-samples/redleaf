use askama::Template;
use axum::{extract::State, response::Response};

use crate::{db::DbPool, models::{Post, Setting}, util::render};

pub mod admin;
pub mod auth;
pub mod posts;

pub use admin::admin_login_routes;
pub use admin::admin_routes;
pub use auth::auth_routes;
pub use posts::post_routes;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    posts: Vec<Post>,
    post_url_type: String,
}

pub async fn index(State(pool): State<DbPool>) -> Response {
    let (posts, post_url_type) = tokio::join!(
        Post::find_all(&pool),
        Setting::post_url_type(&pool),
    );
    let posts = posts.unwrap_or_default();
    render(IndexTemplate { posts, post_url_type })
}