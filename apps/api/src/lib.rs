pub mod config;
pub mod db;
mod routes;

use std::{sync::Arc, time::Instant};

use axum::{
    Json, Router,
    http::{HeaderValue, StatusCode},
};
use serde_json::json;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{config::Config, db::Db};

/// State dùng chung cho mọi handler.
#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub started_at: Arc<Instant>,
}

impl AppState {
    pub fn new(db: Db) -> Self {
        Self {
            db,
            started_at: Arc::new(Instant::now()),
        }
    }
}

pub fn build_app(config: &Config, state: AppState) -> Router {
    let origins: Vec<HeaderValue> = config
        .web_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(Any)
        .allow_headers(Any);

    routes::router()
        .fallback(|| async { (StatusCode::NOT_FOUND, Json(json!({ "error": "not_found" }))) })
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
