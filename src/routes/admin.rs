use axum::{
    extract::{Form, Path, State},
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use serde::Deserialize;

use crate::{
    db::DbPool,
    models::{
        Post,
        post::{CreatePost, UpdatePost},
    },
};

#[derive(Debug, Deserialize)]
struct PostForm {
    title: String,
    slug: String,
    content: String,
    excerpt: Option<String>,
    published: Option<String>, // "on" when checked, absent when not
}

pub fn admin_routes() -> Router<DbPool> {
    Router::new()
        .route("/", get(dashboard))
        .route("/posts", get(list_posts).post(create_post))
        .route("/posts/new", get(new_post_form))
        .route("/posts/{id}", post(update_post))
        .route("/posts/{id}/edit", get(edit_post_form))
        .route("/posts/{id}/delete", post(delete_post))
        .route("/posts/{id}/toggle", post(toggle_published))
}

// ─── Layout ──────────────────────────────────────────────────────────────────

fn layout(title: &str, body: &str) -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title} — RedLeaf Admin</title>
  <style>
    *, *::before, *::after {{ box-sizing: border-box; }}
    body {{ margin: 0; font-family: system-ui, -apple-system, sans-serif; background: #f0f4f0; color: #222; }}
    nav {{ background: #2d5f3e; color: white; padding: .75rem 2rem; display: flex; align-items: center; gap: 1.5rem; }}
    nav a {{ color: white; text-decoration: none; font-weight: 500; font-size: .95rem; }}
    nav a:hover {{ text-decoration: underline; }}
    .logo {{ font-size: 1.15rem; font-weight: bold; margin-right: auto; }}
    main {{ max-width: 1100px; margin: 2rem auto; padding: 0 1.5rem; }}
    h1 {{ color: #2d5f3e; margin-top: 0; }}
    .card {{ background: white; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,.08); padding: 1.5rem; margin-bottom: 1.5rem; }}
    .page-header {{ display: flex; align-items: center; justify-content: space-between; margin-bottom: 1.5rem; }}
    .page-header h1 {{ margin: 0; }}
    table {{ width: 100%; border-collapse: collapse; }}
    th, td {{ text-align: left; padding: .6rem .8rem; border-bottom: 1px solid #eee; vertical-align: middle; }}
    th {{ background: #f9fafb; font-size: .82rem; text-transform: uppercase; letter-spacing: .04em; color: #6b7280; }}
    tr:last-child td {{ border-bottom: none; }}
    .badge {{ display: inline-block; padding: .2rem .6rem; border-radius: 4px; font-size: .78rem; font-weight: 600; }}
    .published {{ background: #d1fae5; color: #065f46; }}
    .draft {{ background: #fef3c7; color: #92400e; }}
    .btn {{ display: inline-block; padding: .42rem .85rem; border-radius: 4px; text-decoration: none;
            font-size: .85rem; border: none; cursor: pointer; font-family: inherit; font-weight: 500; }}
    .btn-primary {{ background: #2d5f3e; color: white; }}
    .btn-primary:hover {{ background: #234a30; }}
    .btn-secondary {{ background: #e5e7eb; color: #374151; }}
    .btn-secondary:hover {{ background: #d1d5db; }}
    .btn-danger {{ background: #fee2e2; color: #991b1b; }}
    .btn-danger:hover {{ background: #fecaca; }}
    .btn-warning {{ background: #fef3c7; color: #92400e; }}
    .btn-warning:hover {{ background: #fde68a; }}
    .acts {{ display: flex; gap: .35rem; align-items: center; flex-wrap: wrap; }}
    .acts form {{ margin: 0; }}
    .form-group {{ margin-bottom: 1.2rem; }}
    label {{ display: block; margin-bottom: .3rem; font-weight: 500; font-size: .9rem; color: #374151; }}
    input[type=text], textarea {{
      width: 100%; padding: .5rem .75rem; border: 1px solid #d1d5db; border-radius: 4px;
      font-family: inherit; font-size: .95rem; background: white;
    }}
    input[type=text]:focus, textarea:focus {{
      outline: none; border-color: #2d5f3e; box-shadow: 0 0 0 3px rgba(45,95,62,.12);
    }}
    textarea.content {{ min-height: 320px; resize: vertical; font-family: ui-monospace, monospace; font-size: .88rem; }}
    .check-group {{ display: flex; align-items: center; gap: .5rem; }}
    .check-group input {{ width: auto; }}
    .check-group label {{ margin: 0; }}
    .stats {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); gap: 1rem; margin-bottom: 1.5rem; }}
    .stat {{ background: white; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,.08);
             padding: 1.25rem 1.5rem; }}
    .stat-value {{ font-size: 2rem; font-weight: 700; color: #2d5f3e; }}
    .stat-label {{ font-size: .85rem; color: #6b7280; margin-top: .2rem; }}
    .empty {{ text-align: center; color: #9ca3af; padding: 2rem; }}
  </style>
</head>
<body>
  <nav>
    <span class="logo">🌿 RedLeaf</span>
    <a href="/admin">Dashboard</a>
    <a href="/admin/posts">Posts</a>
    <a href="/admin/posts/new">New Post</a>
  </nav>
  <main>{body}</main>
</body>
</html>"#
    ))
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn slugify(s: &str) -> String {
    let slug: String = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    slug.split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn post_form(action: &str, post: Option<&Post>) -> String {
    let title = post.map(|p| escape_html(&p.title)).unwrap_or_default();
    let slug = post.map(|p| escape_html(&p.slug)).unwrap_or_default();
    let content = post.map(|p| escape_html(&p.content)).unwrap_or_default();
    let excerpt = post
        .and_then(|p| p.excerpt.as_deref())
        .map(escape_html)
        .unwrap_or_default();
    let checked = if post.map(|p| p.published).unwrap_or(false) {
        "checked"
    } else {
        ""
    };
    let heading = if post.is_some() { "Edit Post" } else { "New Post" };

    format!(
        r#"<div class="page-header">
    <h1>{heading}</h1>
    <a href="/admin/posts" class="btn btn-secondary">← Back to Posts</a>
  </div>
  <div class="card">
    <form method="post" action="{action}">
      <div class="form-group">
        <label for="title">Title <span style="color:#dc2626">*</span></label>
        <input type="text" id="title" name="title" value="{title}" required autocomplete="off">
      </div>
      <div class="form-group">
        <label for="slug">Slug <span style="color:#dc2626">*</span></label>
        <input type="text" id="slug" name="slug" value="{slug}" required autocomplete="off">
      </div>
      <div class="form-group">
        <label for="excerpt">Excerpt</label>
        <input type="text" id="excerpt" name="excerpt" value="{excerpt}" placeholder="Brief summary (optional)">
      </div>
      <div class="form-group">
        <label for="content">Content (Markdown) <span style="color:#dc2626">*</span></label>
        <textarea id="content" name="content" class="content" required>{content}</textarea>
      </div>
      <div class="form-group">
        <div class="check-group">
          <input type="checkbox" id="published" name="published" {checked}>
          <label for="published">Publish immediately</label>
        </div>
      </div>
      <button type="submit" class="btn btn-primary">Save Post</button>
    </form>
  </div>
  <script>
    const titleEl = document.getElementById('title');
    const slugEl  = document.getElementById('slug');
    let slugEdited = slugEl.value.length > 0;
    slugEl.addEventListener('input', () => {{ slugEdited = true; }});
    titleEl.addEventListener('input', () => {{
      if (!slugEdited) {{
        slugEl.value = titleEl.value.toLowerCase()
          .replace(/[^a-z0-9\s-]/g, '')
          .trim()
          .replace(/[\s]+/g, '-');
      }}
    }});
  </script>"#
    )
}

// ─── Handlers ────────────────────────────────────────────────────────────────

async fn dashboard(State(pool): State<DbPool>) -> Html<String> {
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM posts")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    let published = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM posts WHERE published = 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    let drafts = total - published;

    layout(
        "Dashboard",
        &format!(
            r#"<div class="page-header"><h1>Dashboard</h1></div>
  <div class="stats">
    <div class="stat"><div class="stat-value">{total}</div><div class="stat-label">Total Posts</div></div>
    <div class="stat"><div class="stat-value">{published}</div><div class="stat-label">Published</div></div>
    <div class="stat"><div class="stat-value">{drafts}</div><div class="stat-label">Drafts</div></div>
  </div>
  <div class="card">
    <a href="/admin/posts/new" class="btn btn-primary">New Post</a>
    <a href="/admin/posts" class="btn btn-secondary" style="margin-left:.5rem">All Posts</a>
  </div>"#
        ),
    )
}

async fn list_posts(State(pool): State<DbPool>) -> Html<String> {
    let posts = match Post::find_all_admin(&pool).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to fetch posts: {}", e);
            return layout("Posts", r#"<p style="color:#dc2626">Error loading posts.</p>"#);
        }
    };

    let rows: String = if posts.is_empty() {
        r#"<tr><td colspan="6" class="empty">No posts yet. <a href="/admin/posts/new">Create one!</a></td></tr>"#
            .to_string()
    } else {
        posts
            .iter()
            .map(|p| {
                let status = if p.published {
                    r#"<span class="badge published">Published</span>"#
                } else {
                    r#"<span class="badge draft">Draft</span>"#
                };
                let (toggle_label, toggle_class) = if p.published {
                    ("Unpublish", "btn-warning")
                } else {
                    ("Publish", "btn-secondary")
                };
                format!(
                    r#"<tr>
            <td style="color:#9ca3af;font-size:.85rem">{}</td>
            <td><strong>{}</strong></td>
            <td style="color:#6b7280;font-size:.85rem">{}</td>
            <td>{}</td>
            <td style="white-space:nowrap;font-size:.85rem">{}</td>
            <td>
              <div class="acts">
                <a href="/admin/posts/{}/edit" class="btn btn-secondary">Edit</a>
                <form method="post" action="/admin/posts/{}/toggle">
                  <button type="submit" class="btn {}">{}</button>
                </form>
                <form method="post" action="/admin/posts/{}/delete"
                      onsubmit="return confirm('Delete &quot;{}&quot;?')">
                  <button type="submit" class="btn btn-danger">Delete</button>
                </form>
              </div>
            </td>
          </tr>"#,
                    p.id,
                    escape_html(&p.title),
                    escape_html(&p.slug),
                    status,
                    p.created_at.format("%Y-%m-%d"),
                    p.id,
                    p.id,
                    toggle_class,
                    toggle_label,
                    p.id,
                    escape_html(&p.title),
                )
            })
            .collect()
    };

    layout(
        "Posts",
        &format!(
            r#"<div class="page-header">
    <h1>Posts</h1>
    <a href="/admin/posts/new" class="btn btn-primary">New Post</a>
  </div>
  <div class="card">
    <table>
      <thead><tr>
        <th>#</th><th>Title</th><th>Slug</th><th>Status</th><th>Created</th><th>Actions</th>
      </tr></thead>
      <tbody>{rows}</tbody>
    </table>
  </div>"#
        ),
    )
}

async fn new_post_form() -> Html<String> {
    layout("New Post", &post_form("/admin/posts", None))
}

async fn create_post(
    State(pool): State<DbPool>,
    Form(form): Form<PostForm>,
) -> Result<Redirect, Html<String>> {
    let slug = if form.slug.trim().is_empty() {
        slugify(&form.title)
    } else {
        form.slug.trim().to_string()
    };

    let payload = CreatePost {
        title: form.title,
        slug,
        content: form.content,
        excerpt: form.excerpt.filter(|s| !s.trim().is_empty()),
        published: form.published.is_some(),
    };

    Post::create(&pool, payload).await.map_err(|e| {
        tracing::error!("Failed to create post: {}", e);
        layout("New Post", &format!(r#"<p style="color:#dc2626;margin-bottom:1rem">Error: {}</p>{}"#, e, post_form("/admin/posts", None)))
    })?;

    Ok(Redirect::to("/admin/posts"))
}

async fn edit_post_form(State(pool): State<DbPool>, Path(id): Path<i64>) -> Html<String> {
    match Post::find_by_id(&pool, id).await {
        Ok(Some(post)) => {
            let action = format!("/admin/posts/{id}");
            layout("Edit Post", &post_form(&action, Some(&post)))
        }
        Ok(None) => layout("Error", "<p>Post not found.</p>"),
        Err(e) => {
            tracing::error!("Failed to fetch post {}: {}", id, e);
            layout("Error", "<p>Error loading post.</p>")
        }
    }
}

async fn update_post(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Form(form): Form<PostForm>,
) -> Result<Redirect, Html<String>> {
    let slug = if form.slug.trim().is_empty() {
        slugify(&form.title)
    } else {
        form.slug.trim().to_string()
    };

    let payload = UpdatePost {
        title: Some(form.title),
        slug: Some(slug),
        content: Some(form.content),
        excerpt: Some(form.excerpt.filter(|s| !s.trim().is_empty())),
        published: Some(form.published.is_some()),
    };

    match Post::update(&pool, id, payload).await {
        Ok(_) => Ok(Redirect::to("/admin/posts")),
        Err(sqlx::Error::RowNotFound) => Err(layout("Error", "<p>Post not found.</p>")),
        Err(e) => {
            tracing::error!("Failed to update post {}: {}", id, e);
            Err(layout(
                "Error",
                &format!(r#"<p style="color:#dc2626">Error: {e}</p>"#),
            ))
        }
    }
}

async fn delete_post(State(pool): State<DbPool>, Path(id): Path<i64>) -> Redirect {
    if let Err(e) = Post::delete(&pool, id).await {
        tracing::error!("Failed to delete post {}: {}", id, e);
    }
    Redirect::to("/admin/posts")
}

async fn toggle_published(State(pool): State<DbPool>, Path(id): Path<i64>) -> Redirect {
    if let Ok(Some(post)) = Post::find_by_id(&pool, id).await {
        let payload = UpdatePost {
            title: None,
            slug: None,
            content: None,
            excerpt: None,
            published: Some(!post.published),
        };
        if let Err(e) = Post::update(&pool, id, payload).await {
            tracing::error!("Failed to toggle post {}: {}", id, e);
        }
    }
    Redirect::to("/admin/posts")
}