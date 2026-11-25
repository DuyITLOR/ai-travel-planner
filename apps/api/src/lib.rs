pub mod auth;
pub mod config;
pub mod db;
pub mod plans;
pub mod problem;
mod routes;

use std::{sync::Arc, time::Instant};

use axum::{
    Json, Router,
    http::{HeaderName, HeaderValue, StatusCode, header},
};
use serde_json::json;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    auth::JwtVerifier,
    config::Config,
    db::Db,
    plans::store::{InMemoryPlanStore, PlanStore},
};

/// State dùng chung cho mọi handler.
#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub started_at: Arc<Instant>,
    pub plans: Arc<dyn PlanStore>,
    pub auth: JwtVerifier,
}

impl AppState {
    /// Mặc định: plan lưu trong bộ nhớ, chưa có JWT secret nên mọi request cần đăng nhập nhận 401.
    pub fn new(db: Db) -> Self {
        Self {
            db,
            started_at: Arc::new(Instant::now()),
            plans: Arc::new(InMemoryPlanStore::default()),
            auth: JwtVerifier::new(None),
        }
    }

    pub fn with_plans(mut self, plans: Arc<dyn PlanStore>) -> Self {
        self.plans = plans;
        self
    }

    pub fn with_auth(mut self, auth: JwtVerifier) -> Self {
        self.auth = auth;
        self
    }
}

pub fn build_app(config: &Config, state: AppState) -> Router {
    let origins: Vec<HeaderValue> = config
        .web_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();
    // `allow_headers(Any)` answers `*`, which browsers never apply to `Authorization`:
    // name the request headers, and expose the response headers the web client reads.
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(Any)
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            HeaderName::from_static("idempotency-key"),
        ])
        .expose_headers([
            HeaderName::from_static("idempotent-replayed"),
            header::WWW_AUTHENTICATE,
        ]);

    routes::router()
        .fallback(|| async { (StatusCode::NOT_FOUND, Json(json!({ "error": "not_found" }))) })
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
