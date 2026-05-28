use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::auth::validate_token;

// ─── Capability system ────────────────────────────────────────────────────────

static ROLE_CAPS: &[(&str, &[&str])] = &[
    ("administrator", &[
        "read", "edit_posts", "delete_posts", "publish_posts",
        "edit_pages", "delete_pages", "publish_pages",
        "upload_files", "manage_categories", "manage_tags",
        "manage_options", "manage_users", "list_users",
        "edit_others_posts", "delete_others_posts",
    ]),
    ("editor", &[
        "read", "edit_posts", "delete_posts", "publish_posts",
        "edit_pages", "delete_pages", "publish_pages",
        "upload_files", "manage_categories", "manage_tags",
        "edit_others_posts", "delete_others_posts",
    ]),
    ("author", &[
        "read", "edit_posts", "delete_posts", "publish_posts",
        "upload_files",
    ]),
    ("contributor", &[
        "read", "edit_posts", "delete_posts",
    ]),
    ("subscriber", &["read"]),
];

pub fn has_capability(role: &str, cap: &str) -> bool {
    ROLE_CAPS
        .iter()
        .find(|(r, _)| *r == role)
        .map(|(_, caps)| caps.contains(&cap))
        .unwrap_or(false)
}

pub async fn require_auth(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let token = extract_token(&req);

    let token = match token {
        Some(t) => t,
        None => return Ok(redirect_to_login()),
    };

    match validate_token(&token) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(_) => Ok(redirect_to_login()),
    }
}

fn extract_token(req: &Request) -> Option<String> {
    // Try Authorization: Bearer <token> header first
    if let Some(bearer) = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        return Some(bearer.to_owned());
    }

    // Fall back to session cookie
    req.headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|c| {
                c.trim().strip_prefix("session=").map(|s| s.to_owned())
            })
        })
}

fn redirect_to_login() -> Response {
    axum::response::Redirect::to("/admin/login").into_response()
}