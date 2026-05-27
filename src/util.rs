use askama::Template;
use axum::response::{Html, IntoResponse, Response};

use crate::errors::AppError;

pub const PER_PAGE: i64 = 10;

pub fn render<T: Template>(tmpl: T) -> Result<Response, AppError> {
    let html = tmpl.render()?;
    Ok(Html(html).into_response())
}

pub struct Pagination {
    pub current_page: i64,
    pub total_pages: i64,
    pub has_prev: bool,
    pub has_next: bool,
    pub prev_page: i64,
    pub next_page: i64,
    pub total: i64,
    pub base_url: String,
}

impl Pagination {
    pub fn new(page: i64, total: i64, per_page: i64, base_url: impl Into<String>) -> Self {
        let total_pages = if total == 0 { 1 } else { (total + per_page - 1) / per_page };
        let current_page = page.clamp(1, total_pages);
        Self {
            current_page,
            total_pages,
            has_prev: current_page > 1,
            has_next: current_page < total_pages,
            prev_page: (current_page - 1).max(1),
            next_page: (current_page + 1).min(total_pages),
            total,
            base_url: base_url.into(),
        }
    }
}