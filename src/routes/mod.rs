use askama::Template;
use axum::{
    extract::{Form, Path, Query, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::{
    auth::generate_token,
    db::DbPool,
    errors::AppError,
    filters,
    models::{Page, Post, PostWithAuthor, Setting, User, user::CreateUser},
    util::{build_fts_query, render},
};

pub mod admin;
pub mod api;
pub mod auth;
pub mod posts;
pub mod taxonomy;

pub use admin::admin_login_routes;
pub use admin::admin_routes;
pub use auth::auth_routes;
pub use posts::post_routes;
pub use taxonomy::taxonomy_routes;

// ─── Templates ───────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "themes/default/home.html")]
struct IndexTemplate {
    posts: Vec<PostWithAuthor>,
    post_url_type: String,
    site_name: String,
    site_description: String,
    logo_url: String,
}

#[derive(Template)]
#[template(path = "themes/default/search.html")]
struct SearchTemplate {
    query: String,
    posts: Vec<Post>,
    post_url_type: String,
    site_name: String,
}

#[derive(Template)]
#[template(path = "setup.html")]
struct SetupTemplate {
    error: Option<String>,
}

// ─── Handlers ────────────────────────────────────────────────────────────────

pub async fn index(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let (posts, post_url_type, site_name, site_description, logo_url) = tokio::join!(
        Post::find_recent_with_author(&pool, 10),
        Setting::post_url_type(&pool),
        Setting::site_name(&pool),
        Setting::site_description(&pool),
        Setting::logo_url(&pool),
    );
    render(IndexTemplate { posts: posts?, post_url_type, site_name, site_description, logo_url })
}

pub async fn health() -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// ─── Search ──────────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
pub struct SearchQuery {
    pub q: Option<String>,
}

pub async fn search_page(
    State(pool): State<DbPool>,
    Query(params): Query<SearchQuery>,
) -> Result<Response, AppError> {
    let raw = params.q.as_deref().unwrap_or("").trim().to_string();
    let fts = build_fts_query(&raw);

    let (posts, post_url_type, site_name) = tokio::join!(
        async {
            if fts.is_empty() {
                Ok(vec![])
            } else {
                Post::search(&pool, &fts).await
            }
        },
        Setting::post_url_type(&pool),
        Setting::site_name(&pool),
    );

    render(SearchTemplate {
        query: raw,
        posts: posts?,
        post_url_type,
        site_name,
    })
}

// ─── Static page ──────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "themes/default/page.html")]
struct PageShowTemplate {
    page: Page,
    site_name: String,
    html_content: String,
}

pub async fn show_page(
    State(pool): State<DbPool>,
    Path(slug): Path<String>,
) -> Result<Response, AppError> {
    use crate::routes::posts::markdown_to_html_pub;
    let (page, site_name) = tokio::join!(
        Page::find_by_slug(&pool, &slug),
        Setting::site_name(&pool),
    );
    let page = page?.ok_or(AppError::NotFound)?;
    let html_content = markdown_to_html_pub(&page.content);
    render(PageShowTemplate { page, site_name, html_content })
}

// ─── Setup wizard ─────────────────────────────────────────────────────────────

pub async fn setup_page(State(pool): State<DbPool>) -> Result<Response, AppError> {
    if User::has_any(&pool).await {
        return Ok(Redirect::to("/admin/login").into_response());
    }
    render(SetupTemplate { error: None })
}

pub async fn setup_submit(
    State(pool): State<DbPool>,
    Form(form): Form<CreateUser>,
) -> Result<Response, AppError> {
    if User::has_any(&pool).await {
        return Ok(Redirect::to("/admin/login").into_response());
    }

    if form.username.trim().is_empty() || form.email.trim().is_empty() || form.password.len() < 8 {
        return render(SetupTemplate {
            error: Some("Please fill in all fields. Password must be at least 8 characters.".into()),
        });
    }

    let user = User::create(&pool, form).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let token = generate_token(&user)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let cookie = format!(
        "session={}; HttpOnly; Path=/; SameSite=Strict; Max-Age=604800",
        token
    );
    Ok(axum::response::Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin")
        .header(header::SET_COOKIE, cookie)
        .body(axum::body::Body::empty())
        .unwrap())
}

// ─── 404 fallback ─────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "themes/default/404.html")]
struct NotFoundTemplate {
    site_name: String,
}

pub async fn not_found_handler(State(pool): State<DbPool>) -> impl IntoResponse {
    let site_name = Setting::site_name(&pool).await;
    match (NotFoundTemplate { site_name }).render() {
        Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}