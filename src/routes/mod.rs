use askama::Template;
use axum::{
    extract::{Form, Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use std::collections::HashMap;

use crate::{
    auth::generate_token,
    db::DbPool,
    errors::AppError,
    filters,
    models::{NavMenu, Page, Post, PostWithAuthor, Setting, User, user::CreateUser},
    util::{build_fts_query, render, Pagination, PER_PAGE},
};

pub mod admin;
pub mod api;
pub mod auth;
pub mod feed;
pub mod posts;
pub mod taxonomy;

pub use admin::admin_login_routes;
pub use admin::admin_routes;
pub use auth::auth_routes;
pub use feed::{feed_routes, category_feed_route};
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
    nav_menus: HashMap<String, String>,
}

impl IndexTemplate {
    fn render_nav_menu(&self, location: &str) -> &str {
        self.nav_menus.get(location).map(|s| s.as_str()).unwrap_or("")
    }
}

#[derive(Template)]
#[template(path = "themes/default/search.html")]
struct SearchTemplate {
    query: String,
    posts: Vec<Post>,
    post_url_type: String,
    site_name: String,
    nav_menus: HashMap<String, String>,
}

impl SearchTemplate {
    fn render_nav_menu(&self, location: &str) -> &str {
        self.nav_menus.get(location).map(|s| s.as_str()).unwrap_or("")
    }
}

#[derive(Template)]
#[template(path = "setup.html")]
struct SetupTemplate {
    error: Option<String>,
}

// ─── Handlers ────────────────────────────────────────────────────────────────

pub async fn index(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let (posts, post_url_type, site_name, site_description, logo_url, nav_menus) = tokio::join!(
        Post::find_recent_with_author(&pool, 10),
        Setting::post_url_type(&pool),
        Setting::site_name(&pool),
        Setting::site_description(&pool),
        Setting::logo_url(&pool),
        NavMenu::prerender_all(&pool),
    );
    render(IndexTemplate { posts: posts?, post_url_type, site_name, site_description, logo_url, nav_menus })
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

    let (posts, post_url_type, site_name, nav_menus) = tokio::join!(
        async {
            if fts.is_empty() {
                Ok(vec![])
            } else {
                Post::search(&pool, &fts).await
            }
        },
        Setting::post_url_type(&pool),
        Setting::site_name(&pool),
        NavMenu::prerender_all(&pool),
    );

    render(SearchTemplate {
        query: raw,
        posts: posts?,
        post_url_type,
        site_name,
        nav_menus,
    })
}

// ─── Static page ──────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "themes/default/page.html")]
struct PageShowTemplate {
    page: Page,
    site_name: String,
    html_content: String,
    nav_menus: HashMap<String, String>,
}

impl PageShowTemplate {
    fn render_nav_menu(&self, location: &str) -> &str {
        self.nav_menus.get(location).map(|s| s.as_str()).unwrap_or("")
    }

    fn the_breadcrumb(&self) -> String {
        use crate::models::nav_menu::{BreadcrumbItem, breadcrumb_html};
        let items = vec![
            BreadcrumbItem { label: "Home".into(), url: Some("/".into()) },
            BreadcrumbItem { label: self.page.title.clone(), url: None },
        ];
        breadcrumb_html(&items)
    }

    fn breadcrumb_json_ld(&self) -> String {
        use crate::models::nav_menu::{BreadcrumbItem, breadcrumb_json_ld};
        let items = vec![
            BreadcrumbItem { label: "Home".into(), url: Some("/".into()) },
            BreadcrumbItem { label: self.page.title.clone(), url: Some(format!("/pages/{}", self.page.slug)) },
        ];
        breadcrumb_json_ld(&items)
    }
}

#[derive(Template)]
#[template(path = "themes/default/404.html")]
struct NotFoundTemplate {
    site_name: String,
    nav_menus: HashMap<String, String>,
}

impl NotFoundTemplate {
    fn render_nav_menu(&self, location: &str) -> &str {
        self.nav_menus.get(location).map(|s| s.as_str()).unwrap_or("")
    }
}

pub async fn show_page(
    State(pool): State<DbPool>,
    Path(slug): Path<String>,
) -> Result<Response, AppError> {
    use crate::routes::posts::markdown_to_html_pub;
    let (page, site_name, nav_menus) = tokio::join!(
        Page::find_by_slug(&pool, &slug),
        Setting::site_name(&pool),
        NavMenu::prerender_all(&pool),
    );
    let page = page?.ok_or(AppError::NotFound)?;
    let html_content = markdown_to_html_pub(&page.content);
    render(PageShowTemplate { page, site_name, html_content, nav_menus })
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

// ─── Author archive ───────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "themes/default/author.html")]
struct AuthorTemplate {
    author_display_name: String,
    author_username: String,
    author_bio: String,
    author_website: String,
    author_avatar_url: String,
    posts: Vec<PostWithAuthor>,
    paging: Pagination,
    site_name: String,
    nav_menus: HashMap<String, String>,
}

impl AuthorTemplate {
    fn render_nav_menu(&self, location: &str) -> &str {
        self.nav_menus.get(location).map(|s| s.as_str()).unwrap_or("")
    }

    fn author_initial(&self) -> String {
        self.author_display_name
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "?".into())
    }

    fn the_breadcrumb(&self) -> String {
        use crate::models::nav_menu::{BreadcrumbItem, breadcrumb_html};
        let items = vec![
            BreadcrumbItem { label: "Home".into(), url: Some("/".into()) },
            BreadcrumbItem { label: self.author_display_name.clone(), url: None },
        ];
        breadcrumb_html(&items)
    }

    fn breadcrumb_json_ld(&self) -> String {
        use crate::models::nav_menu::{BreadcrumbItem, breadcrumb_json_ld};
        let items = vec![
            BreadcrumbItem { label: "Home".into(), url: Some("/".into()) },
            BreadcrumbItem {
                label: self.author_display_name.clone(),
                url: Some(format!("/author/{}", self.author_username)),
            },
        ];
        breadcrumb_json_ld(&items)
    }
}

#[derive(Deserialize, Default)]
pub struct AuthorQuery {
    pub page: Option<i64>,
}

pub async fn author_page(
    State(pool): State<DbPool>,
    Path(username): Path<String>,
    Query(q): Query<AuthorQuery>,
) -> Result<Response, AppError> {
    let user = User::find_by_username(&pool, &username).await?
        .ok_or(AppError::NotFound)?;

    let page = q.page.unwrap_or(1).max(1);
    let (posts, total, site_name, nav_menus) = tokio::join!(
        Post::find_by_author_paginated(&pool, user.id, page, PER_PAGE),
        Post::count_by_author(&pool, user.id),
        Setting::site_name(&pool),
        NavMenu::prerender_all(&pool),
    );
    let total = total?;
    let paging = Pagination::new(page, total, PER_PAGE, format!("/author/{}", username));

    let display_name = user.effective_display_name().to_string();
    render(AuthorTemplate {
        author_display_name: display_name,
        author_username: user.username,
        author_bio: user.bio,
        author_website: user.website,
        author_avatar_url: user.avatar_url,
        posts: posts?,
        paging,
        site_name,
        nav_menus,
    })
}

// ─── 404 fallback ─────────────────────────────────────────────────────────────

pub async fn not_found_handler(State(pool): State<DbPool>) -> impl IntoResponse {
    let (site_name, nav_menus) = tokio::join!(
        Setting::site_name(&pool),
        NavMenu::prerender_all(&pool),
    );
    match (NotFoundTemplate { site_name, nav_menus }).render() {
        Ok(html) => (StatusCode::NOT_FOUND, Html(html)).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

// ─── Sitemap ──────────────────────────────────────────────────────────────────

fn host_from_headers(headers: &HeaderMap) -> String {
    headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost")
        .to_string()
}

fn base_from_host(host: &str) -> String {
    let scheme = if host.starts_with("localhost") || host.starts_with("127.") { "http" } else { "https" };
    format!("{}://{}", scheme, host)
}

pub async fn sitemap_xml(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let host = host_from_headers(&headers);
    let base = base_from_host(&host);

    let (posts, pages) = tokio::join!(
        Post::find_all_admin(&pool),
        Page::find_all(&pool),
    );

    let mut urls = String::new();
    if let Ok(posts) = posts {
        for p in posts.iter().filter(|p| p.published) {
            let loc = format!("{}/posts/{}", base, xml_escape(&p.slug));
            let lastmod = p.updated_at.format("%Y-%m-%d").to_string();
            urls.push_str(&format!(
                "<url><loc>{loc}</loc><lastmod>{lastmod}</lastmod><changefreq>monthly</changefreq><priority>0.8</priority></url>\n"
            ));
        }
    }
    if let Ok(pages) = pages {
        for p in pages.iter().filter(|p| p.status == "published") {
            let loc = format!("{}/pages/{}", base, xml_escape(&p.slug));
            urls.push_str(&format!(
                "<url><loc>{loc}</loc><changefreq>monthly</changefreq><priority>0.6</priority></url>\n"
            ));
        }
    }

    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
<url><loc>{base}/</loc><changefreq>daily</changefreq><priority>1.0</priority></url>
{urls}</urlset>"#
    );
    (
        [(axum::http::header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        body,
    )
}

pub async fn sitemap_index_xml(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let _ = pool;
    let host = host_from_headers(&headers);
    let base = base_from_host(&host);
    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
<sitemap><loc>{base}/sitemap.xml</loc><lastmod>{now}</lastmod></sitemap>
</sitemapindex>"#
    );
    (
        [(axum::http::header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        body,
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

// ─── robots.txt ───────────────────────────────────────────────────────────────

pub async fn robots_txt(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let host = host_from_headers(&headers);
    let base = base_from_host(&host);
    let sitemap_url = format!("{}/sitemap.xml", base);

    let custom = Setting::get(&pool, "robots_txt").await.unwrap_or_default();
    let body = if custom.trim().is_empty() {
        format!("User-agent: *\nAllow: /\n\nSitemap: {sitemap_url}\n")
    } else {
        // Append sitemap line if not already present
        if custom.contains("Sitemap:") {
            custom
        } else {
            format!("{}\n\nSitemap: {sitemap_url}\n", custom.trim())
        }
    };
    (
        [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        body,
    )
}