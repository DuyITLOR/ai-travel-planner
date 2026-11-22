mod health;

use axum::{Router, routing::get};

use crate::AppState;

/// Gắn route của từng tính năng tại đây (places, itineraries, groups, ...).
pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health::health))
}
