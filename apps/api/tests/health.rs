use atp_api::{AppState, build_app, config::Config, db::Db};
use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

fn app() -> Router {
    let config = Config {
        port: 0,
        web_origins: vec!["http://localhost:3000".into()],
        database_url: None,
    };
    build_app(&config, AppState::new(Db::disabled()))
}

async fn get(path: &str, origin: Option<&str>) -> (StatusCode, HeaderMap, Value) {
    let mut req = Request::get(path);
    if let Some(o) = origin {
        req = req.header("origin", o);
    }
    let res = app()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let (parts, body) = res.into_parts();
    let bytes = body.collect().await.unwrap().to_bytes();
    (
        parts.status,
        parts.headers,
        serde_json::from_slice(&bytes).unwrap(),
    )
}

#[tokio::test]
async fn health_reports_db_not_configured() {
    let (status, _, body) = get("/health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["db"], "not_configured");
    assert!(body["uptimeSec"].is_u64());
}

#[tokio::test]
async fn cors_allows_web_origin() {
    let (_, headers, _) = get("/health", Some("http://localhost:3000")).await;
    assert_eq!(
        headers["access-control-allow-origin"],
        "http://localhost:3000"
    );
}

#[tokio::test]
async fn unknown_route_returns_json_404() {
    let (status, _, body) = get("/nope", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "not_found");
}
