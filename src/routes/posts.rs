use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};

use crate::{db::DbPool, models::Post};

pub fn post_routes() -> Router<DbPool> {
    Router::new()
        .route("/", get(list_posts))
        .route("/{id}", get(show_post))
}

// List all posts
async fn list_posts(State(pool): State<DbPool>) -> Html<String> {
    match Post::find_all(&pool).await {
        Ok(posts) => {
            let posts_html: String = posts.iter()
                .map(|post| format!(r#"
                    <div class="post">
                        <h2><a href="/posts/{}">{}</a></h2>
                        <p class="meta">Published: {}</p>
                        <p>{}</p>
                    </div>
                "#, post.id, post.title, post.created_at,
                   post.content.chars().take(200).collect::<String>()))
                .collect();

            Html(format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Posts - RedLeaf CMS</title>
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            background: #f5f5f5;
        }}
        .post {{
            background: white;
            padding: 1.5rem;
            margin-bottom: 1.5rem;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .post h2 {{
            margin-top: 0;
            color: #2d5f3e;
        }}
        .post a {{
            text-decoration: none;
            color: inherit;
        }}
        .post a:hover {{
            text-decoration: underline;
        }}
        .meta {{
            color: #666;
            font-size: 0.9rem;
        }}
    </style>
</head>
<body>
    <h1>🌿 All Posts</h1>
    {}
</body>
</html>
            "#, posts_html))
        }
        Err(e) => {
            tracing::error!("Failed to fetch posts: {}", e);
            Html(format!("<h1>Error loading posts: {}</h1>", e))
        }
    }
}

// Show a single post
async fn show_post(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Html<String> {
    match Post::find_by_id(&pool, id).await {
        Ok(Some(post)) => {
            let html_content = markdown_to_html(&post.content);

            Html(format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - RedLeaf CMS</title>
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            background: #f5f5f5;
        }}
        .post {{
            background: white;
            padding: 2rem;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .post h1 {{
            margin-top: 0;
            color: #2d5f3e;
        }}
        .meta {{
            color: #666;
            font-size: 0.9rem;
            margin-bottom: 2rem;
            padding-bottom: 1rem;
            border-bottom: 1px solid #eee;
        }}
        .content {{
            line-height: 1.6;
        }}
        .back {{
            margin-bottom: 1rem;
        }}
        .back a {{
            color: #2d5f3e;
            text-decoration: none;
        }}
        .back a:hover {{
            text-decoration: underline;
        }}
    </style>
</head>
<body>
    <div class="back">
        <a href="/posts">← Back to all posts</a>
    </div>
    <div class="post">
        <h1>{}</h1>
        <p class="meta">Published: {}</p>
        <div class="content">
            {}
        </div>
    </div>
</body>
</html>
            "#, post.title, post.title, post.created_at, html_content))
        }
        Ok(None) => Html("<h1>Post not found</h1>".to_string()),
        Err(e) => {
            tracing::error!("Failed to fetch post: {}", e);
            Html(format!("<h1>Error loading post: {}</h1>", e))
        }
    }
}

// Convert markdown to HTML
fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{Parser, Options, html};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}
