//! In-memory page cache implemented as a Tower middleware via
//! `axum::middleware::from_fn`. Caches successful GET responses for public
//! HTML/XML/feed URLs and serves ETag / Last-Modified conditional-request
//! semantics to save bandwidth.

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use bytes::Bytes;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

/// Maximum age of a cache entry (5 min).
const CACHE_TTL: Duration = Duration::from_secs(300);
/// Refuse to buffer responses larger than 4 MB.
const MAX_BODY: usize = 4 * 1024 * 1024;

// ─── Cache store ──────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Entry {
    body: Bytes,
    status: StatusCode,
    content_type: String,
    etag: String,
    last_modified: String,
    born: Instant,
}

pub struct PageCache {
    store: RwLock<HashMap<String, Entry>>,
}

impl PageCache {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            store: RwLock::new(HashMap::new()),
        })
    }

    /// Returns `true` when this path should be considered for caching.
    pub fn is_cacheable(path: &str) -> bool {
        !path.starts_with("/admin")
            && !path.starts_with("/api")
            && !path.starts_with("/auth")
            && !path.starts_with("/setup")
            && !path.starts_with("/static")
            && path != "/health"
    }

    fn get(&self, key: &str) -> Option<Entry> {
        let store = self.store.read().ok()?;
        let e = store.get(key)?;
        if e.born.elapsed() > CACHE_TTL {
            return None;
        }
        Some(e.clone())
    }

    fn insert(&self, key: String, body: Bytes, status: StatusCode, content_type: String) {
        let etag = etag_of(&body);
        let last_modified = http_date_now();
        if let Ok(mut store) = self.store.write() {
            store.insert(key, Entry { body, status, content_type, etag, last_modified, born: Instant::now() });
        }
    }

    /// Remove all cached entries — call on any content mutation.
    pub fn invalidate_all(&self) {
        if let Ok(mut store) = self.store.write() {
            store.clear();
        }
    }
}

// ─── Tower middleware ──────────────────────────────────────────────────────────

/// Tower-compatible middleware (via `axum::middleware::from_fn`).
/// The `PageCache` is injected through the closure in `build_app`, so no
/// additional extractor is needed.
pub async fn middleware(cache: Arc<PageCache>, request: Request, next: Next) -> Response {
    // Only cache idempotent, side-effect-free requests.
    if request.method() != Method::GET {
        return next.run(request).await;
    }

    let path = request.uri().path().to_string();
    if !PageCache::is_cacheable(&path) {
        return next.run(request).await;
    }

    let query = request.uri().query()
        .map(|q| format!("?{q}"))
        .unwrap_or_default();
    let key = format!("{path}{query}");

    // Capture conditional headers before the request is consumed.
    let if_none_match = header_str(request.headers(), "if-none-match");
    let _if_modified_since = header_str(request.headers(), "if-modified-since");

    // ── Serve from cache ───────────────────────────────────────────────────
    if let Some(entry) = cache.get(&key) {
        // Conditional GET: ETag match → 304 Not Modified.
        if let Some(ref inm) = if_none_match {
            if inm.trim() == entry.etag.as_str() || inm.trim() == "*" {
                return not_modified(&entry.etag, &entry.last_modified);
            }
        }
        return hit_response(entry);
    }

    // ── Forward to handler ─────────────────────────────────────────────────
    let response = next.run(request).await;
    let status = response.status();
    if !status.is_success() {
        return response;
    }

    let (parts, body) = response.into_parts();
    let content_type = parts.headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    // Only cache textual content (HTML, XML, RSS/Atom).
    let cacheable_type = content_type.contains("text/html")
        || content_type.contains("text/xml")
        || content_type.contains("application/xml")
        || content_type.contains("application/rss")
        || content_type.contains("application/atom");

    if !cacheable_type {
        return Response::from_parts(parts, body);
    }

    let body_bytes = match axum::body::to_bytes(body, MAX_BODY).await {
        Ok(b) => b,
        Err(_) => return Response::from_parts(parts, Body::empty()),
    };

    let etag = etag_of(&body_bytes);
    let last_modified = http_date_now();
    cache.insert(key, body_bytes.clone(), status, content_type.clone());

    // Attach cache-control / ETag / Last-Modified to the fresh response.
    let mut resp = Response::from_parts(parts, Body::from(body_bytes));
    set_header(&mut resp, "etag", &etag);
    set_header(&mut resp, "last-modified", &last_modified);
    resp.headers_mut().insert("cache-control", HeaderValue::from_static("public, max-age=300"));
    resp.headers_mut().insert("x-cache", HeaderValue::from_static("MISS"));
    resp
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn etag_of(data: &[u8]) -> String {
    let mut h = DefaultHasher::new();
    data.hash(&mut h);
    format!("\"{:016x}\"", h.finish())
}

fn http_date_now() -> String {
    chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

fn header_str(headers: &axum::http::HeaderMap, name: &str) -> Option<String> {
    headers.get(name)?.to_str().ok().map(|s| s.to_string())
}

fn not_modified(etag: &str, last_modified: &str) -> Response {
    let mut r = Response::new(Body::empty());
    *r.status_mut() = StatusCode::NOT_MODIFIED;
    set_header(&mut r, "etag", etag);
    set_header(&mut r, "last-modified", last_modified);
    r
}

fn hit_response(e: Entry) -> Response {
    let mut r = Response::new(Body::from(e.body));
    *r.status_mut() = e.status;
    set_header(&mut r, "content-type", &e.content_type);
    set_header(&mut r, "etag", &e.etag);
    set_header(&mut r, "last-modified", &e.last_modified);
    r.headers_mut().insert("cache-control", HeaderValue::from_static("public, max-age=300"));
    r.headers_mut().insert("x-cache", HeaderValue::from_static("HIT"));
    r
}

fn set_header(resp: &mut Response, name: &'static str, value: &str) {
    if let Ok(v) = HeaderValue::from_str(value) {
        resp.headers_mut().insert(name, v);
    }
}