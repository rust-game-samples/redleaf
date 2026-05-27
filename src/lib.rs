pub mod auth;
pub mod db;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod util;

use axum::{middleware as axum_middleware, routing::get, Router};
use tower_http::{services::ServeDir, trace::TraceLayer};

pub fn build_app(pool: db::DbPool) -> Router {
    let protected_admin = routes::admin_routes()
        .layer(axum_middleware::from_fn(middleware::require_auth));

    Router::new()
        .route("/", get(routes::index))
        .nest("/posts", routes::post_routes())
        .nest("/admin", protected_admin)
        .merge(routes::admin_login_routes())
        .nest("/auth", routes::auth_routes())
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}