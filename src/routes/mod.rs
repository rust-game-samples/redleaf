use askama::Template;
use axum::{extract::State, response::Response};

use crate::{
    db::DbPool,
    errors::AppError,
    models::{Post, Setting},
    util::render,
};

pub mod admin;
pub mod api;
pub mod auth;
pub mod posts;
pub mod taxonomy;

pub use admin::admin_login_routes;
pub use admin::admin_routes;
pub use auth::auth_routes;
pub use posts::post_routes;
pub use taxonomy::taxonomy_routes;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    posts: Vec<Post>,
    post_url_type: String,
    site_name: String,
    site_description: String,
    logo_url: String,
}

pub async fn index(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let (posts, post_url_type, site_name, site_description, logo_url) = tokio::join!(
        Post::find_all(&pool),
        Setting::post_url_type(&pool),
        Setting::site_name(&pool),
        Setting::site_description(&pool),
        Setting::logo_url(&pool),
    );
    render(IndexTemplate { posts: posts?, post_url_type, site_name, site_description, logo_url })
}