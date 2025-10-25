use axum::{
    response::Html,
    routing::get,
    Router,
};

use crate::db::DbPool;

pub fn admin_routes() -> Router<DbPool> {
    Router::new()
        .route("/", get(admin_dashboard))
}

// Admin dashboard
async fn admin_dashboard() -> Html<String> {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Admin Dashboard - RedLeaf CMS</title>
    <style>
        body {
            font-family: system-ui, -apple-system, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
            background: #f5f5f5;
        }
        .header {
            background: white;
            padding: 1.5rem;
            margin-bottom: 2rem;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .header h1 {
            margin: 0;
            color: #2d5f3e;
        }
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 1.5rem;
        }
        .card {
            background: white;
            padding: 1.5rem;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .card h2 {
            margin-top: 0;
            color: #333;
        }
        .actions {
            margin-top: 1rem;
        }
        .btn {
            display: inline-block;
            padding: 0.5rem 1rem;
            margin-right: 0.5rem;
            background: #2d5f3e;
            color: white;
            text-decoration: none;
            border-radius: 4px;
        }
        .btn:hover {
            background: #234a30;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🌿 RedLeaf Admin Dashboard</h1>
    </div>
    <div class="grid">
        <div class="card">
            <h2>Posts</h2>
            <p>Manage your blog posts</p>
            <div class="actions">
                <a href="/admin/posts/new" class="btn">New Post</a>
                <a href="/admin/posts" class="btn">All Posts</a>
            </div>
        </div>
        <div class="card">
            <h2>Settings</h2>
            <p>Configure your CMS</p>
            <div class="actions">
                <a href="/admin/settings" class="btn">Settings</a>
            </div>
        </div>
        <div class="card">
            <h2>Statistics</h2>
            <p>View your site statistics</p>
            <div class="actions">
                <a href="/admin/stats" class="btn">View Stats</a>
            </div>
        </div>
    </div>
</body>
</html>
    "#.to_string())
}
