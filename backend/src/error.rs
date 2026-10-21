//! ApiError 與統一錯誤格式

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::judge::JudgeError;

/// JUDGE_BUSY 建議重試秒數
const RETRY_AFTER_SECS: &str = "2";

#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    InvalidRequest(String),
    LevelNotFound(String),
    PayloadTooLarge,
    JudgeBusy,
    JudgeUnavailable(String),
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: String,
}

impl From<JudgeError> for ApiError {
    fn from(e: JudgeError) -> Self {
        match e {
            JudgeError::Busy => Self::JudgeBusy,
            JudgeError::Unavailable(reason) => Self::JudgeUnavailable(reason),
            JudgeError::Internal(detail) => Self::Internal(detail),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::InvalidRequest(detail) => (StatusCode::BAD_REQUEST, "INVALID_REQUEST", detail),
            Self::LevelNotFound(id) => (
                StatusCode::NOT_FOUND,
                "LEVEL_NOT_FOUND",
                format!("找不到關卡：{id}"),
            ),
            Self::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "PAYLOAD_TOO_LARGE",
                "請求內容超過上限".to_string(),
            ),
            Self::JudgeBusy => (
                StatusCode::SERVICE_UNAVAILABLE,
                "JUDGE_BUSY",
                "目前判題人數過多，請稍後再試".to_string(),
            ),
            Self::JudgeUnavailable(reason) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "JUDGE_UNAVAILABLE",
                format!("判題服務未就緒：{reason}"),
            ),
            Self::Internal(detail) => {
                tracing::error!("內部錯誤：{detail}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "伺服器內部錯誤".to_string(),
                )
            }
        };
        let mut resp = (
            status,
            Json(ErrorBody {
                error: ErrorDetail { code, message },
            }),
        )
            .into_response();
        if code == "JUDGE_BUSY" {
            resp.headers_mut().insert(
                header::RETRY_AFTER,
                HeaderValue::from_static(RETRY_AFTER_SECS),
            );
        }
        resp
    }
}
