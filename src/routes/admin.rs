use askama::Template;
use axum::{
    extract::{Extension, Form, Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;

use crate::{
    auth::{generate_token, Claims},
    db::DbPool,
    errors::AppError,
    models::{
        Post,
        PostWithAuthor,
        Setting,
        post::{CreatePost, UpdatePost},
        user::{CreateUser, LoginUser},
        User,
    },
    util::{render, Pagination, PER_PAGE},
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
    posts: Vec<PostWithAuthor>,
    paging: Pagination,
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

#[derive(Deserialize, Default)]
struct PageQuery {
    page: Option<i64>,
}

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

async fn login_page() -> Result<Response, AppError> {
    render(LoginTemplate { error: None })
}

async fn login_submit(
    State(pool): State<DbPool>,
    Form(payload): Form<LoginUser>,
) -> Result<Response, AppError> {
    let user = match User::authenticate(&pool, payload)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        Some(u) => u,
        None => {
            return render(LoginTemplate {
                error: Some("Invalid email or password.".to_string()),
            });
        }
    };

    let token = generate_token(&user)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let cookie = format!(
        "session={}; HttpOnly; Path=/; SameSite=Strict; Max-Age=604800",
        token
    );
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin")
        .header(header::SET_COOKIE, cookie)
        .body(axum::body::Body::empty())
        .unwrap())
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

async fn register_page() -> Result<Response, AppError> {
    render(RegisterTemplate {
        error: None,
        prefill_username: String::new(),
        prefill_email: String::new(),
    })
}

async fn register_submit(
    State(pool): State<DbPool>,
    Form(payload): Form<CreateUser>,
) -> Result<Response, AppError> {
    let prefill_username = payload.username.clone();
    let prefill_email = payload.email.clone();

    let user = match User::create(&pool, payload).await {
        Ok(u) => u,
        Err(e) => {
            let msg = if e.to_string().contains("UNIQUE constraint failed") {
                "Username or email is already taken.".to_string()
            } else {
                return Err(AppError::Internal(e.to_string()));
            };
            return render(RegisterTemplate {
                error: Some(msg),
                prefill_username,
                prefill_email,
            });
        }
    };

    let token = generate_token(&user)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let cookie = format!(
        "session={}; HttpOnly; Path=/; SameSite=Strict; Max-Age=604800",
        token
    );
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin")
        .header(header::SET_COOKIE, cookie)
        .body(axum::body::Body::empty())
        .unwrap())
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
        author_id: None,
    }
}

// ─── Handlers ────────────────────────────────────────────────────────────────

async fn dashboard(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let (total, published) = tokio::join!(
        Post::count_all(&pool),
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM posts WHERE published = 1")
            .fetch_one(&pool),
    );
    let total = total?;
    let published = published?;
    render(DashboardTemplate { total, published, drafts: total - published })
}

async fn list_posts(
    State(pool): State<DbPool>,
    Query(q): Query<PageQuery>,
) -> Result<Response, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let (posts, total) = tokio::join!(
        Post::find_admin_paginated(&pool, page, PER_PAGE),
        Post::count_all(&pool),
    );
    let paging = Pagination::new(page, total?, PER_PAGE, "/admin/posts");
    render(PostListTemplate { posts: posts?, paging })
}

async fn new_post_form() -> Result<Response, AppError> {
    render(PostFormTemplate::new("/admin/posts", "New Post"))
}

async fn create_post(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Form(form): Form<PostForm>,
) -> Result<Response, AppError> {
    let mut payload = form_to_create(form);
    payload.author_id = Some(claims.sub);

    match Post::create(&pool, payload).await {
        Ok(_) => Ok(Redirect::to("/admin/posts").into_response()),
        Err(e) => {
            let msg = if e.to_string().contains("UNIQUE constraint failed") {
                "Slug already exists. Please choose a different slug.".to_string()
            } else {
                return Err(AppError::Database(e));
            };
            let mut tmpl = PostFormTemplate::new("/admin/posts", "New Post");
            tmpl.error = Some(msg);
            render(tmpl)
        }
    }
}

async fn edit_post_form(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let post = Post::find_by_id(&pool, id).await?.ok_or(AppError::NotFound)?;
    render(PostFormTemplate::from_post(&post))
}

async fn update_post(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Form(form): Form<PostForm>,
) -> Result<Response, AppError> {
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

    Post::update(&pool, id, payload).await.map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound,
        other => AppError::Database(other),
    })?;
    Ok(Redirect::to("/admin/posts").into_response())
}

async fn delete_post(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    Post::delete(&pool, id).await?;
    Ok(Redirect::to("/admin/posts").into_response())
}

async fn toggle_published(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let post = Post::find_by_id(&pool, id).await?.ok_or(AppError::NotFound)?;
    let payload = UpdatePost {
        title: None,
        slug: None,
        content: None,
        excerpt: None,
        published: Some(!post.published),
    };
    Post::update(&pool, id, payload).await?;
    Ok(Redirect::to("/admin/posts").into_response())
}

// ─── Settings ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SettingsForm {
    post_url_type: Option<String>,
}

async fn settings_page(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let post_url_type = Setting::post_url_type(&pool).await;
    render(SettingsTemplate { post_url_type, saved: None })
}

async fn settings_save(
    State(pool): State<DbPool>,
    Form(form): Form<SettingsForm>,
) -> Result<Response, AppError> {
    let url_type = match form.post_url_type.as_deref() {
        Some("id") => "id",
        _ => "slug",
    };
    Setting::set(&pool, "post_url_type", url_type).await?;
    render(SettingsTemplate {
        post_url_type: url_type.to_string(),
        saved: Some("Settings saved.".to_string()),
    })
}