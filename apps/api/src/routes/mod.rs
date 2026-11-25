mod health;
mod plans;

use axum::{
    Router,
    routing::{get, post},
};

use crate::AppState;

/// Gắn route của từng tính năng tại đây (places, itineraries, groups, ...).
pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health::health)).route(
        "/plans/{plan_id}/save",
        post(plans::save_plan).fallback(plans::method_not_allowed),
    )
}
