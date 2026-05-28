use askama::Template;
use axum::{
    extract::{Form, Path, Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Deserializer};

use std::collections::HashMap;

use crate::{
    db::DbPool,
    errors::AppError,
    filters,
    models::{Comment, NavMenu, Post, PostWithAuthor, Setting, Tag, Widget},
    util::{render, Pagination, PER_PAGE},
};

#[derive(Template)]
#[template(path = "themes/default/archive.html")]
struct PostListTemplate {
    posts: Vec<Post>,
    post_url_type: String,
    paging: Pagination,
    site_name: String,
    widget_areas: HashMap<String, String>,
    nav_menus: HashMap<String, String>,
}

impl PostListTemplate {
    fn permalink_for(&self, post: &Post) -> String {
        if self.post_url_type == "id" {
            format!("/posts/{}", post.id)
        } else {
            format!("/posts/{}", post.slug)
        }
    }

    fn home_url(&self) -> &str {
        "/"
    }

    fn site_url(&self) -> &str {
        "/"
    }

    fn render_widget_area(&self, slug: &str) -> &str {
        self.widget_areas.get(slug).map(|s| s.as_str()).unwrap_or("")
    }

    fn render_nav_menu(&self, location: &str) -> &str {
        self.nav_menus.get(location).map(|s| s.as_str()).unwrap_or("")
    }

    fn the_breadcrumb(&self) -> String {
        use crate::models::nav_menu::{BreadcrumbItem, breadcrumb_html};
        let items = vec![
            BreadcrumbItem { label: "Home".into(), url: Some("/".into()) },
            BreadcrumbItem { label: "Posts".into(), url: None },
        ];
        breadcrumb_html(&items)
    }
}

#[derive(Template)]
#[template(path = "themes/default/single.html")]
struct PostShowTemplate {
    post: PostWithAuthor,
    html_content: String,
    tags: Vec<Tag>,
    comments: Vec<Comment>,
    comment_count: i64,
    comment_notice: Option<String>,
    site_name: String,
    post_url_type: String,
    widget_areas: HashMap<String, String>,
    nav_menus: HashMap<String, String>,
}

impl PostShowTemplate {
    /// `the_title` — returns the post title (HTML-escaped by Askama automatically).
    fn the_title(&self) -> &str {
        &self.post.title
    }

    /// `the_content` — returns rendered HTML content (mark as safe in template with `|safe`).
    fn the_content(&self) -> &str {
        &self.html_content
    }

    /// `the_excerpt` — returns the stored excerpt, or auto-generates one from content.
    fn the_excerpt(&self) -> String {
        if let Some(exc) = &self.post.excerpt {
            if !exc.is_empty() {
                return exc.clone();
            }
        }
        let plain = crate::filters::strip_markdown(&self.post.content);
        if plain.chars().count() <= 150 {
            plain
        } else {
            let t: String = plain.chars().take(150).collect();
            let cut = t.rfind(' ').unwrap_or(t.len());
            format!("{}…", &t[..cut])
        }
    }

    /// `the_permalink` — canonical URL for this post.
    fn the_permalink(&self) -> String {
        if self.post_url_type == "id" {
            format!("/posts/{}", self.post.id)
        } else {
            format!("/posts/{}", self.post.slug)
        }
    }

    /// `the_date` — publication date in the given strftime format.
    fn the_date(&self, fmt: &str) -> String {
        self.post.created_at.format(fmt).to_string()
    }

    /// `the_author` — display name of the post author, if available.
    fn the_author(&self) -> &str {
        self.post
            .author_username
            .as_deref()
            .unwrap_or("")
    }

    /// `the_post_thumbnail` — renders an `<img>` tag for the featured image.
    /// `size` is one of: "thumbnail" (150px), "medium" (300px), "large" (1024px), or "full".
    fn the_post_thumbnail(&self, size: &str) -> String {
        let url = match &self.post.featured_image_url {
            Some(u) => u,
            None => return String::new(),
        };
        let max_w = match size {
            "thumbnail" => "150px",
            "medium" => "300px",
            "large" => "1024px",
            _ => "100%",
        };
        let alt = escape_html(&self.post.title);
        format!(
            r#"<img src="{url}" alt="{alt}" style="max-width:{max_w};object-fit:cover;" loading="lazy">"#
        )
    }

    fn home_url(&self) -> &str {
        "/"
    }

    fn site_url(&self) -> &str {
        "/"
    }

    fn render_widget_area(&self, slug: &str) -> &str {
        self.widget_areas.get(slug).map(|s| s.as_str()).unwrap_or("")
    }

    fn render_nav_menu(&self, location: &str) -> &str {
        self.nav_menus.get(location).map(|s| s.as_str()).unwrap_or("")
    }

    fn the_breadcrumb(&self) -> String {
        use crate::models::nav_menu::{BreadcrumbItem, breadcrumb_html};
        let mut items = vec![
            BreadcrumbItem { label: "Home".into(), url: Some("/".into()) },
        ];
        if let (Some(cat_name), Some(cat_slug)) = (&self.post.category_name, &self.post.category_slug) {
            items.push(BreadcrumbItem { label: cat_name.clone(), url: Some(format!("/categories/{}", cat_slug)) });
        }
        items.push(BreadcrumbItem { label: self.post.title.clone(), url: None });
        breadcrumb_html(&items)
    }

    fn breadcrumb_json_ld(&self) -> String {
        use crate::models::nav_menu::{BreadcrumbItem, breadcrumb_json_ld};
        let mut items = vec![
            BreadcrumbItem { label: "Home".into(), url: Some("/".into()) },
        ];
        if let (Some(cat_name), Some(cat_slug)) = (&self.post.category_name, &self.post.category_slug) {
            items.push(BreadcrumbItem { label: cat_name.clone(), url: Some(format!("/categories/{}", cat_slug)) });
        }
        items.push(BreadcrumbItem { label: self.post.title.clone(), url: Some(self.the_permalink()) });
        breadcrumb_json_ld(&items)
    }

    fn render_comments(&self) -> String {
        Comment::render_thread(&self.comments, None)
    }

    fn effective_seo_title(&self) -> &str {
        if !self.post.seo_title.is_empty() { &self.post.seo_title } else { &self.post.title }
    }

    fn effective_seo_description(&self) -> String {
        if !self.post.seo_description.is_empty() {
            self.post.seo_description.clone()
        } else {
            self.the_excerpt()
        }
    }

    fn article_json_ld(&self) -> String {
        let title = self.post.title.replace('"', "\\\"");
        let desc = self.effective_seo_description().replace('"', "\\\"");
        let url = self.the_permalink();
        let date_pub = self.post.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let date_mod = self.post.updated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let author = self.the_author();
        let img = self.post.featured_image_url.as_deref().unwrap_or("");
        format!(
            r#"<script type="application/ld+json">{{
"@context":"https://schema.org",
"@type":"Article",
"headline":"{title}",
"description":"{desc}",
"url":"{url}",
"datePublished":"{date_pub}",
"dateModified":"{date_mod}",
"author":{{"@type":"Person","name":"{author}"}},
"image":"{img}"
}}</script>"#
        )
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[derive(Deserialize, Default)]
struct PageQuery {
    page: Option<i64>,
    comment: Option<String>,
}

pub fn post_routes() -> Router<DbPool> {
    Router::new()
        .route("/", get(list_posts))
        .route("/{param}", get(show_post))
        .route("/{param}/comment", post(submit_comment))
}

async fn list_posts(
    State(pool): State<DbPool>,
    Query(q): Query<PageQuery>,
) -> Result<Response, AppError> {
    let page = q.page.unwrap_or(1).max(1);

    let (posts, total, post_url_type, site_name, widget_areas, nav_menus) = tokio::join!(
        Post::find_published_paginated(&pool, page, PER_PAGE),
        Post::count_published(&pool),
        Setting::post_url_type(&pool),
        Setting::site_name(&pool),
        Widget::prerender_all(&pool),
        NavMenu::prerender_all(&pool),
    );

    let paging = Pagination::new(page, total?, PER_PAGE, "/posts");
    render(PostListTemplate { posts: posts?, post_url_type, paging, site_name, widget_areas, nav_menus })
}

async fn show_post(
    State(pool): State<DbPool>,
    Path(param): Path<String>,
    Query(q): Query<PageQuery>,
) -> Result<Response, AppError> {
    let (url_type, site_name, widget_areas, nav_menus) = tokio::join!(
        Setting::post_url_type(&pool),
        Setting::site_name(&pool),
        Widget::prerender_all(&pool),
        NavMenu::prerender_all(&pool),
    );

    let post = if url_type == "id" {
        let id = param
            .parse::<i64>()
            .map_err(|_| AppError::NotFound)?;
        Post::find_by_id_with_author(&pool, id).await?
    } else {
        Post::find_by_slug_with_author(&pool, &param).await?
    };

    let post = post.ok_or(AppError::NotFound)?;
    let (tags, comments, comment_count) = tokio::join!(
        Tag::find_by_post(&pool, post.id),
        Comment::find_by_post(&pool, post.id),
        Comment::count_by_post(&pool, post.id),
    );
    let html_content = markdown_to_html(&post.content);
    let comment_notice = match q.comment.as_deref() {
        Some("pending") => Some("Your comment has been submitted and is awaiting moderation.".into()),
        Some("error") => Some("There was a problem submitting your comment. Please try again.".into()),
        _ => None,
    };
    render(PostShowTemplate {
        post,
        html_content,
        tags: tags?,
        comments: comments?,
        comment_count: comment_count?,
        comment_notice,
        site_name,
        post_url_type: url_type,
        widget_areas,
        nav_menus,
    })
}

fn empty_string_as_none<'de, D>(d: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(d)?;
    match s.as_deref() {
        None | Some("") => Ok(None),
        Some(v) => v.parse::<i64>().map(Some).map_err(serde::de::Error::custom),
    }
}

#[derive(Deserialize)]
struct CommentForm {
    author_name: String,
    author_email: Option<String>,
    content: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    parent_id: Option<i64>,
}

async fn submit_comment(
    State(pool): State<DbPool>,
    Path(param): Path<String>,
    Form(form): Form<CommentForm>,
) -> Result<Response, AppError> {
    let url_type = Setting::post_url_type(&pool).await;
    let post = if url_type == "id" {
        let id = param.parse::<i64>().map_err(|_| AppError::NotFound)?;
        Post::find_by_id_with_author(&pool, id).await?
    } else {
        Post::find_by_slug_with_author(&pool, &param).await?
    };
    let post = post.ok_or(AppError::NotFound)?;
    let redirect_base = if url_type == "id" {
        format!("/posts/{}", post.id)
    } else {
        format!("/posts/{}", post.slug)
    };

    let name = form.author_name.trim();
    let content = form.content.trim();
    if name.is_empty() || content.is_empty() {
        return Ok(Redirect::to(&format!("{}?comment=error", redirect_base)).into_response());
    }

    let email = form.author_email.as_deref().unwrap_or("").trim();
    match Comment::create(&pool, post.id, form.parent_id, name, email, content).await {
        Ok(_) => Ok(Redirect::to(&format!("{}?comment=pending", redirect_base)).into_response()),
        Err(_) => Ok(Redirect::to(&format!("{}?comment=error", redirect_base)).into_response()),
    }
}

pub fn markdown_to_html_pub(markdown: &str) -> String {
    markdown_to_html(markdown)
}

fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};

    let normalized = markdown.replace("\r\n", "\n");

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(&normalized, options);
    let mut output = String::new();
    html::push_html(&mut output, parser);
    output
}