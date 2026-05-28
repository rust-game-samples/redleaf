use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use crate::{
    db::DbPool,
    models::{Post, PostWithAuthor, Setting},
};

pub fn feed_routes() -> Router<DbPool> {
    Router::new()
        .route("/feed", get(rss_feed))
        .route("/feed/atom", get(atom_feed))
}

pub fn category_feed_route() -> Router<DbPool> {
    Router::new()
        .route("/categories/{slug}/feed", get(category_rss_feed))
}

// ─── Helper ───────────────────────────────────────────────────────────────────

fn base_url(headers: &HeaderMap) -> String {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");
    let scheme = if host.starts_with("localhost") || host.starts_with("127.") {
        "http"
    } else {
        "https"
    };
    format!("{}://{}", scheme, host)
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn rfc2822(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%a, %d %b %Y %H:%M:%S +0000").to_string()
}

fn post_excerpt(post: &PostWithAuthor) -> String {
    if let Some(exc) = &post.excerpt {
        if !exc.is_empty() {
            return exc.clone();
        }
    }
    let content = &post.content;
    let plain: String = content
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join(" ");
    let plain = plain.trim().to_string();
    if plain.chars().count() <= 200 {
        plain
    } else {
        let t: String = plain.chars().take(200).collect();
        let cut = t.rfind(' ').unwrap_or(t.len());
        format!("{}…", &t[..cut])
    }
}

fn build_rss(
    base: &str,
    site_name: &str,
    site_description: &str,
    feed_url: &str,
    posts: &[PostWithAuthor],
) -> String {
    let mut items = String::new();
    for post in posts.iter().take(20) {
        let url = format!("{}/posts/{}", base, escape_xml(&post.slug));
        let title = escape_xml(&post.title);
        let desc = escape_xml(&post_excerpt(post));
        let pub_date = rfc2822(&post.created_at);
        items.push_str(&format!(
            "<item>\
<title>{title}</title>\
<link>{url}</link>\
<guid isPermaLink=\"true\">{url}</guid>\
<pubDate>{pub_date}</pubDate>\
<description>{desc}</description>\
</item>\n"
        ));
    }
    let site_name = escape_xml(site_name);
    let site_desc = escape_xml(site_description);
    let feed_url = escape_xml(feed_url);
    let base_esc = escape_xml(base);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
<channel>
<title>{site_name}</title>
<link>{base_esc}</link>
<description>{site_desc}</description>
<language>en</language>
<atom:link href="{feed_url}" rel="self" type="application/rss+xml"/>
{items}</channel>
</rss>"#
    )
}

fn build_atom(
    base: &str,
    site_name: &str,
    feed_url: &str,
    posts: &[PostWithAuthor],
) -> String {
    let updated = posts
        .first()
        .map(|p| p.updated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());

    let mut entries = String::new();
    for post in posts.iter().take(20) {
        let url = format!("{}/posts/{}", base, escape_xml(&post.slug));
        let title = escape_xml(&post.title);
        let summary = escape_xml(&post_excerpt(post));
        let updated_dt = post.updated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        entries.push_str(&format!(
            "<entry>\
<title>{title}</title>\
<link href=\"{url}\"/>\
<id>{url}</id>\
<updated>{updated_dt}</updated>\
<summary>{summary}</summary>\
</entry>\n"
        ));
    }
    let site_name = escape_xml(site_name);
    let feed_url = escape_xml(feed_url);
    let base_esc = escape_xml(base);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
<title>{site_name}</title>
<link href="{base_esc}"/>
<link rel="self" href="{feed_url}"/>
<updated>{updated}</updated>
<id>{base_esc}/</id>
{entries}</feed>"#
    )
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

pub async fn rss_feed(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let base = base_url(&headers);
    let (posts, site_name, site_description) = tokio::join!(
        Post::find_published_paginated_with_author(&pool, 1, 20),
        Setting::site_name(&pool),
        Setting::site_description(&pool),
    );
    let posts = posts.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let feed_url = format!("{}/feed", base);
    let body = build_rss(&base, &site_name, &site_description, &feed_url, &posts);
    Ok((
        [(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        body,
    )
        .into_response())
}

pub async fn atom_feed(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let base = base_url(&headers);
    let (posts, site_name) = tokio::join!(
        Post::find_published_paginated_with_author(&pool, 1, 20),
        Setting::site_name(&pool),
    );
    let posts = posts.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let feed_url = format!("{}/feed/atom", base);
    let body = build_atom(&base, &site_name, &feed_url, &posts);
    Ok((
        [(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")],
        body,
    )
        .into_response())
}

pub async fn category_rss_feed(
    State(pool): State<DbPool>,
    Path(slug): Path<String>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let base = base_url(&headers);
    let (posts, site_name, site_description) = tokio::join!(
        Post::find_by_category(&pool, &slug, 1, 20),
        Setting::site_name(&pool),
        Setting::site_description(&pool),
    );
    let posts = posts.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let feed_url = format!("{}/categories/{}/feed", base, slug);
    let cat_name = slug.replace('-', " ");
    let title = format!("{} — {}", cat_name, site_name);
    let body = build_rss(&base, &title, &site_description, &feed_url, &posts);
    Ok((
        [(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        body,
    )
        .into_response())
}