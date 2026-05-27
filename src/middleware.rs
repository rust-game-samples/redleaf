use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::auth::validate_token;

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