//! GET /api/health

use axum::Json;
use axum::extract::State;
use serde::Serialize;

use crate::AppState;
use crate::judge::UnavailableReason;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    judge: JudgeStatusBody,
}

#[derive(Serialize)]
struct JudgeStatusBody {
    available: bool,
    reason: Option<&'static str>,
}

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let h = state.judge.health().await;
    Json(HealthResponse {
        status: if h.available { "ok" } else { "degraded" },
        judge: JudgeStatusBody {
            available: h.available,
            reason: h.reason.map(|r| match r {
                UnavailableReason::DockerUnreachable => "docker_unreachable",
                UnavailableReason::RunnerImageMissing => "runner_image_missing",
            }),
        },
    })
}
