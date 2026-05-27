use askama::Template;
use axum::{extract::State, response::Response};

use crate::{db::DbPool, models::Post, util::render};

pub mod admin;
pub mod auth;
pub mod posts;

pub use admin::admin_routes;
pub use admin::admin_login_routes;
pub use auth::auth_routes;
pub use posts::post_routes;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    posts: Vec<Post>,
}

pub async fn index(State(pool): State<DbPool>) -> Response {
    let posts = Post::find_all(&pool).await.unwrap_or_default();
    render(IndexTemplate { posts })
}