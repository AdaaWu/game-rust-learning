//! Router 組裝

mod health;
mod judge;
mod levels;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderValue, header};
use axum::middleware::map_response;
use axum::response::Response;
use axum::routing::{get, post};

use crate::AppState;

/// request body 上限
const BODY_LIMIT_BYTES: usize = 128 * 1024;

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/api/levels", get(levels::list))
        .route("/api/levels/{id}", get(levels::detail))
        .route("/api/levels/{id}/run", post(judge::run))
        .route("/api/levels/{id}/submit", post(judge::submit))
        .route("/api/health", get(health::health))
        .layer(DefaultBodyLimit::max(BODY_LIMIT_BYTES))
        .layer(map_response(json_charset))
        .with_state(state)
}

/// JSON 回應的 Content-Type 補上 charset=utf-8
async fn json_charset(mut resp: Response) -> Response {
    let headers = resp.headers_mut();
    if headers.get(header::CONTENT_TYPE).map(HeaderValue::as_bytes) == Some(b"application/json") {
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=utf-8"),
        );
    }
    resp
}
