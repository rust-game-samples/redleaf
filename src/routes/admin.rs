use askama::Template;
use axum::{
    extract::{Form, Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;

use crate::{
    auth::generate_token,
    db::DbPool,
    models::{
        Post,
        Setting,
        post::{CreatePost, UpdatePost},
        user::{CreateUser, LoginUser},
        User,
    },
    util::render,
};

// ─── Login / register templates ──────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/login.html")]
struct LoginTemplate {
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/register.html")]
struct RegisterTemplate {
    error: Option<String>,
    prefill_username: String,
    prefill_email: String,
}

// ─── Settings template ───────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/settings.html")]
struct SettingsTemplate {
    post_url_type: String,
    saved: Option<String>,
}

// ─── Template structs ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
struct DashboardTemplate {
    total: i64,
    published: i64,
    drafts: i64,
}

#[derive(Template)]
#[template(path = "admin/posts/list.html")]
struct PostListTemplate {
    posts: Vec<Post>,
}

#[derive(Template)]
#[template(path = "admin/posts/form.html")]
struct PostFormTemplate {
    heading: String,
    action: String,
    title: String,
    slug: String,
    content: String,
    excerpt: String,
    published: bool,
    error: Option<String>,
}

impl PostFormTemplate {
    fn new(action: impl Into<String>, heading: impl Into<String>) -> Self {
        Self {
            heading: heading.into(),
            action: action.into(),
            title: String::new(),
            slug: String::new(),
            content: String::new(),
            excerpt: String::new(),
            published: false,
            error: None,
        }
    }

    fn from_post(post: &Post) -> Self {
        Self {
            heading: "Edit Post".into(),
            action: format!("/admin/posts/{}", post.id),
            title: post.title.clone(),
            slug: post.slug.clone(),
            content: post.content.clone(),
            excerpt: post.excerpt.clone().unwrap_or_default(),
            published: post.published,
            error: None,
        }
    }
}

// ─── Form input ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct PostForm {
    title: String,
    slug: String,
    content: String,
    excerpt: Option<String>,
    published: Option<String>,
}

// ─── Router ──────────────────────────────────────────────────────────────────

pub fn admin_login_routes() -> Router<DbPool> {
    Router::new()
        .route("/admin/login", get(login_page).post(login_submit))
        .route("/admin/register", get(register_page).post(register_submit))
        .route("/admin/logout", post(logout))
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
        .route("/settings", get(settings_page).post(settings_save))
}

// ─── Login / logout handlers ─────────────────────────────────────────────────

async fn login_page() -> Response {
    render(LoginTemplate { error: None })
}

async fn login_submit(State(pool): State<DbPool>, Form(payload): Form<LoginUser>) -> Response {
    let user = match User::authenticate(&pool, payload).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return render(LoginTemplate {
                error: Some("Invalid email or password.".to_string()),
            })
        }
        Err(e) => {
            tracing::error!("Authentication error: {}", e);
            return render(LoginTemplate {
                error: Some("Internal error. Please try again.".to_string()),
            });
        }
    };

    let token = match generate_token(&user) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Token generation failed: {}", e);
            return render(LoginTemplate {
                error: Some("Internal error. Please try again.".to_string()),
            });
        }
    };

    let cookie = format!(
        "session={}; HttpOnly; Path=/; SameSite=Strict; Max-Age=604800",
        token
    );
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin")
        .header(header::SET_COOKIE, cookie)
        .body(axum::body::Body::empty())
        .unwrap()
}

async fn logout() -> Response {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin/login")
        .header(
            header::SET_COOKIE,
            "session=; HttpOnly; Path=/; SameSite=Strict; Max-Age=0",
        )
        .body(axum::body::Body::empty())
        .unwrap()
}

async fn register_page() -> Response {
    render(RegisterTemplate {
        error: None,
        prefill_username: String::new(),
        prefill_email: String::new(),
    })
}

async fn register_submit(
    State(pool): State<DbPool>,
    Form(payload): Form<CreateUser>,
) -> Response {
    let prefill_username = payload.username.clone();
    let prefill_email = payload.email.clone();

    let user = match User::create(&pool, payload).await {
        Ok(u) => u,
        Err(e) => {
            let msg = if e.to_string().contains("UNIQUE constraint failed") {
                "Username or email is already taken.".to_string()
            } else {
                tracing::error!("Registration error: {}", e);
                "Internal error. Please try again.".to_string()
            };
            return render(RegisterTemplate {
                error: Some(msg),
                prefill_username,
                prefill_email,
            });
        }
    };

    let token = match generate_token(&user) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Token generation failed: {}", e);
            return render(RegisterTemplate {
                error: Some("Internal error. Please try again.".to_string()),
                prefill_username,
                prefill_email,
            });
        }
    };

    let cookie = format!(
        "session={}; HttpOnly; Path=/; SameSite=Strict; Max-Age=604800",
        token
    );
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin")
        .header(header::SET_COOKIE, cookie)
        .body(axum::body::Body::empty())
        .unwrap()
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

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

fn form_to_create(form: PostForm) -> CreatePost {
    let slug = if form.slug.trim().is_empty() {
        slugify(&form.title)
    } else {
        form.slug.trim().to_string()
    };
    CreatePost {
        title: form.title,
        slug,
        content: form.content,
        excerpt: form.excerpt.filter(|s| !s.trim().is_empty()),
        published: form.published.is_some(),
    }
}

// ─── Handlers ────────────────────────────────────────────────────────────────

async fn dashboard(State(pool): State<DbPool>) -> Response {
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM posts")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    let published =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM posts WHERE published = 1")
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

    render(DashboardTemplate {
        total,
        published,
        drafts: total - published,
    })
}

async fn list_posts(State(pool): State<DbPool>) -> Response {
    match Post::find_all_admin(&pool).await {
        Ok(posts) => render(PostListTemplate { posts }),
        Err(e) => {
            tracing::error!("Failed to fetch posts: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error loading posts").into_response()
        }
    }
}

async fn new_post_form() -> Response {
    render(PostFormTemplate::new("/admin/posts", "New Post"))
}

async fn create_post(State(pool): State<DbPool>, Form(form): Form<PostForm>) -> Response {
    let payload = form_to_create(form);

    match Post::create(&pool, payload).await {
        Ok(_) => Redirect::to("/admin/posts").into_response(),
        Err(e) => {
            tracing::error!("Failed to create post: {}", e);
            let msg = if e.to_string().contains("UNIQUE constraint failed") {
                "Slug already exists. Please choose a different slug.".to_string()
            } else {
                format!("Error saving post: {e}")
            };
            let mut tmpl = PostFormTemplate::new("/admin/posts", "New Post");
            tmpl.error = Some(msg);
            render(tmpl)
        }
    }
}

async fn edit_post_form(State(pool): State<DbPool>, Path(id): Path<i64>) -> Response {
    match Post::find_by_id(&pool, id).await {
        Ok(Some(post)) => render(PostFormTemplate::from_post(&post)),
        Ok(None) => (StatusCode::NOT_FOUND, "Post not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch post {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error loading post").into_response()
        }
    }
}

async fn update_post(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Form(form): Form<PostForm>,
) -> Response {
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
        Ok(_) => Redirect::to("/admin/posts").into_response(),
        Err(sqlx::Error::RowNotFound) => {
            (StatusCode::NOT_FOUND, "Post not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to update post {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {e}")).into_response()
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

// ─── Settings ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SettingsForm {
    post_url_type: Option<String>,
}

async fn settings_page(State(pool): State<DbPool>) -> Response {
    let post_url_type = Setting::post_url_type(&pool).await;
    render(SettingsTemplate { post_url_type, saved: None })
}

async fn settings_save(
    State(pool): State<DbPool>,
    Form(form): Form<SettingsForm>,
) -> Response {
    let url_type = match form.post_url_type.as_deref() {
        Some("id") => "id",
        _ => "slug",
    };

    if let Err(e) = Setting::set(&pool, "post_url_type", url_type).await {
        tracing::error!("Failed to save settings: {}", e);
    }

    render(SettingsTemplate {
        post_url_type: url_type.to_string(),
        saved: Some("Settings saved.".to_string()),
    })
}