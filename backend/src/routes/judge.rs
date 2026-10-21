//! POST /api/levels/:id/run、/api/levels/:id/submit

use axum::Json;
use axum::body::Bytes;
use axum::extract::rejection::BytesRejection;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::AppState;
use crate::error::ApiError;

/// 依序驗證 body 大小、Content-Type、JSON 與 code 欄位、code 大小
fn parse_code(
    state: &AppState,
    headers: &HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Result<String, ApiError> {
    let body = body.map_err(|e| {
        if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
            ApiError::PayloadTooLarge
        } else {
            ApiError::InvalidRequest("無法讀取請求內容".into())
        }
    })?;
    let is_json = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(';').next())
        .is_some_and(|v| v.trim().eq_ignore_ascii_case("application/json"));
    if !is_json {
        return Err(ApiError::InvalidRequest(
            "Content-Type 必須是 application/json".into(),
        ));
    }
    let value: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|_| ApiError::InvalidRequest("請求內容不是合法 JSON".into()))?;
    let code = value
        .get("code")
        .and_then(|c| c.as_str())
        .filter(|c| !c.is_empty())
        .ok_or_else(|| ApiError::InvalidRequest("code 必須是非空字串".into()))?;
    if code.len() > state.judge.config().max_code_bytes {
        return Err(ApiError::PayloadTooLarge);
    }
    Ok(code.to_string())
}

pub async fn run(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Result<Response, ApiError> {
    let code = parse_code(&state, &headers, body)?;
    let level = state.levels.get(&id).ok_or(ApiError::LevelNotFound(id))?;
    let resp = state.judge.run(level, &code).await?;
    Ok(Json(resp).into_response())
}

pub async fn submit(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Result<Response, ApiError> {
    let code = parse_code(&state, &headers, body)?;
    let level = state.levels.get(&id).ok_or(ApiError::LevelNotFound(id))?;
    let resp = state
        .judge
        .submit(level, state.levels.grid_sim(), &code)
        .await?;
    Ok(Json(resp).into_response())
}
