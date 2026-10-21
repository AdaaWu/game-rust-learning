//! GET /api/levels、/api/levels/:id

use axum::Json;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::AppState;
use crate::content::model::{ChapterSummary, LevelDetail, LevelSummary};
use crate::error::ApiError;

#[derive(Serialize)]
struct LevelList<'a> {
    chapters: &'a [ChapterSummary],
    levels: Vec<LevelSummary<'a>>,
}

pub async fn list(State(state): State<AppState>) -> Response {
    let body = LevelList {
        chapters: state.levels.chapters(),
        levels: state
            .levels
            .levels()
            .iter()
            .map(|l| LevelSummary::from(l.as_ref()))
            .collect(),
    };
    Json(body).into_response()
}

pub async fn detail(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    match state.levels.get(&id) {
        Some(level) => Json(LevelDetail::from(level.as_ref())).into_response(),
        None => ApiError::LevelNotFound(id).into_response(),
    }
}
