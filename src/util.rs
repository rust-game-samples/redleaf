use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

pub fn render<T: Template>(tmpl: T) -> Response {
    match tmpl.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Template render failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Template render failed").into_response()
        }
    }
}