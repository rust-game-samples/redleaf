use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::get,
    Router,
};
use serde::Deserialize;

use std::collections::HashMap;

use crate::{
    db::DbPool,
    errors::AppError,
    filters,
    models::{Category, NavMenu, Post, PostWithAuthor, Setting, Tag},
    util::{render, Pagination, PER_PAGE},
};

#[derive(Template)]
#[template(path = "themes/default/taxonomy/category.html")]
struct CategoryPageTemplate {
    category: Category,
    posts: Vec<PostWithAuthor>,
    paging: Pagination,
    site_name: String,
    nav_menus: HashMap<String, String>,
}

impl CategoryPageTemplate {
    fn render_nav_menu(&self, location: &str) -> &str {
        self.nav_menus.get(location).map(|s| s.as_str()).unwrap_or("")
    }

    fn the_breadcrumb(&self) -> String {
        use crate::models::nav_menu::{BreadcrumbItem, breadcrumb_html};
        let items = vec![
            BreadcrumbItem { label: "Home".into(), url: Some("/".into()) },
            BreadcrumbItem { label: "Categories".into(), url: Some("/categories".into()) },
            BreadcrumbItem { label: self.category.name.clone(), url: None },
        ];
        breadcrumb_html(&items)
    }

    fn breadcrumb_json_ld(&self) -> String {
        use crate::models::nav_menu::{BreadcrumbItem, breadcrumb_json_ld};
        let items = vec![
            BreadcrumbItem { label: "Home".into(), url: Some("/".into()) },
            BreadcrumbItem { label: "Categories".into(), url: Some("/categories".into()) },
            BreadcrumbItem { label: self.category.name.clone(), url: Some(format!("/categories/{}", self.category.slug)) },
        ];
        breadcrumb_json_ld(&items)
    }
}

#[derive(Template)]
#[template(path = "themes/default/taxonomy/tag.html")]
struct TagPageTemplate {
    tag: Tag,
    posts: Vec<PostWithAuthor>,
    paging: Pagination,
    site_name: String,
    nav_menus: HashMap<String, String>,
}

impl TagPageTemplate {
    fn render_nav_menu(&self, location: &str) -> &str {
        self.nav_menus.get(location).map(|s| s.as_str()).unwrap_or("")
    }

    fn the_breadcrumb(&self) -> String {
        use crate::models::nav_menu::{BreadcrumbItem, breadcrumb_html};
        let items = vec![
            BreadcrumbItem { label: "Home".into(), url: Some("/".into()) },
            BreadcrumbItem { label: "Tags".into(), url: Some("/tags".into()) },
            BreadcrumbItem { label: self.tag.name.clone(), url: None },
        ];
        breadcrumb_html(&items)
    }

    fn breadcrumb_json_ld(&self) -> String {
        use crate::models::nav_menu::{BreadcrumbItem, breadcrumb_json_ld};
        let items = vec![
            BreadcrumbItem { label: "Home".into(), url: Some("/".into()) },
            BreadcrumbItem { label: "Tags".into(), url: Some("/tags".into()) },
            BreadcrumbItem { label: self.tag.name.clone(), url: Some(format!("/tags/{}", self.tag.slug)) },
        ];
        breadcrumb_json_ld(&items)
    }
}

#[derive(Deserialize, Default)]
struct PageQuery {
    page: Option<i64>,
}

pub fn taxonomy_routes() -> Router<DbPool> {
    Router::new()
        .route("/categories/{slug}", get(category_page))
        .route("/tags/{slug}", get(tag_page))
}

async fn category_page(
    State(pool): State<DbPool>,
    Path(slug): Path<String>,
    Query(q): Query<PageQuery>,
) -> Result<Response, AppError> {
    let category = Category::find_by_slug(&pool, &slug)
        .await?
        .ok_or(AppError::NotFound)?;
    let page = q.page.unwrap_or(1).max(1);
    let (posts, total, site_name, nav_menus) = tokio::join!(
        Post::find_by_category(&pool, &slug, page, PER_PAGE),
        Post::count_by_category(&pool, &slug),
        Setting::site_name(&pool),
        NavMenu::prerender_all(&pool),
    );
    let base = format!("/categories/{}", slug);
    let paging = Pagination::new(page, total?, PER_PAGE, &base);
    render(CategoryPageTemplate { category, posts: posts?, paging, site_name, nav_menus })
}

async fn tag_page(
    State(pool): State<DbPool>,
    Path(slug): Path<String>,
    Query(q): Query<PageQuery>,
) -> Result<Response, AppError> {
    let tag = Tag::find_by_slug(&pool, &slug)
        .await?
        .ok_or(AppError::NotFound)?;
    let page = q.page.unwrap_or(1).max(1);
    let (posts, total, site_name, nav_menus) = tokio::join!(
        Post::find_by_tag(&pool, &slug, page, PER_PAGE),
        Post::count_by_tag(&pool, &slug),
        Setting::site_name(&pool),
        NavMenu::prerender_all(&pool),
    );
    let base = format!("/tags/{}", slug);
    let paging = Pagination::new(page, total?, PER_PAGE, &base);
    render(TagPageTemplate { tag, posts: posts?, paging, site_name, nav_menus })
}