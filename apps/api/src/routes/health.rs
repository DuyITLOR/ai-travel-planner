use axum::{Json, extract::State};
use serde::Serialize;

use crate::{AppState, db::DbStatus};

/// Phải khớp với `HealthResponse` trong packages/shared (web dùng để đọc).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    status: &'static str,
    uptime_sec: u64,
    commit: String,
    db: DbStatus,
}

/// Uptime check và đánh thức Render trước demo.
pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        uptime_sec: state.started_at.elapsed().as_secs(),
        commit: std::env::var("RENDER_GIT_COMMIT")
            .ok()
            .filter(|c| !c.trim().is_empty())
            .map(|c| c.chars().take(7).collect())
            .unwrap_or_else(|| "local".into()),
        db: state.db.ping().await,
    })
}
