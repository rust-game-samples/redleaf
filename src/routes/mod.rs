use axum::{
    extract::State,
    response::Html,
};

use crate::db::DbPool;

pub mod posts;
pub mod admin;

pub use posts::post_routes;
pub use admin::admin_routes;

// Home page handler
pub async fn index(State(_pool): State<DbPool>) -> Html<String> {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RedLeaf CMS</title>
    <style>
        body {
            font-family: system-ui, -apple-system, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            background: #f5f5f5;
        }
        .header {
            text-align: center;
            margin-bottom: 3rem;
        }
        .logo {
            font-size: 3rem;
        }
        h1 {
            color: #2d5f3e;
        }
        .tagline {
            color: #666;
            font-style: italic;
        }
        .features {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-top: 2rem;
        }
        .feature {
            background: white;
            padding: 1.5rem;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
    </style>
</head>
<body>
    <div class="header">
        <div class="logo">🌿</div>
        <h1>RedLeaf CMS</h1>
        <p class="tagline">A lightweight, blazing-fast CMS built with Rust</p>
    </div>
    <div class="features">
        <div class="feature">
            <h3>⚡ Fast</h3>
            <p>Compiled Rust backend with minimal runtime</p>
        </div>
        <div class="feature">
            <h3>🪶 Lightweight</h3>
            <p>Single binary, easy to deploy</p>
        </div>
        <div class="feature">
            <h3>🧱 Extensible</h3>
            <p>Plugin & theme support</p>
        </div>
        <div class="feature">
            <h3>🌐 Headless Ready</h3>
            <p>API-first architecture</p>
        </div>
    </div>
</body>
</html>
    "#.to_string())
}
