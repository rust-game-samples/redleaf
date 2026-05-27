use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "redleaf=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let pool = redleaf::db::init_db().await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Background task: auto-publish scheduled posts every 60 seconds
    let scheduler_pool = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            match redleaf::models::Post::publish_scheduled(&scheduler_pool).await {
                Ok(n) if n > 0 => tracing::info!("Scheduled: published {} post(s)", n),
                Err(e) => tracing::error!("Scheduler error: {}", e),
                _ => {}
            }
        }
    });

    let app = redleaf::build_app(pool);

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("{host}:{port}").parse()?;

    tracing::info!("🌿 RedLeaf CMS listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}