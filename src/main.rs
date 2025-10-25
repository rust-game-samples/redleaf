use axum::{
    Router,
    routing::get,
};
use std::net::SocketAddr;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod models;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "redleaf=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize database
    let db_pool = db::init_db().await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await?;

    // Build application routes
    let app = Router::new()
        .route("/", get(routes::index))
        .nest("/posts", routes::post_routes())
        .nest("/admin", routes::admin_routes())
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .with_state(db_pool);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("🌿 RedLeaf CMS listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
