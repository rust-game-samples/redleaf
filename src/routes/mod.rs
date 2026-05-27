use askama::Template;
use axum::response::Response;

use crate::util::render;

pub mod admin;
pub mod auth;
pub mod posts;

pub use admin::admin_routes;
pub use auth::auth_routes;
pub use posts::post_routes;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

pub async fn index() -> Response {
    render(IndexTemplate)
}