use atp_api::{AppState, auth::JwtVerifier, build_app, config::Config, db::Db};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // .env chỉ dùng khi chạy local; trên Render biến môi trường đặt trong dashboard.
    dotenvy::dotenv().ok();
    fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .ok()
                .filter(|v| !v.trim().is_empty())
                .and_then(|v| EnvFilter::try_new(v).ok())
                .unwrap_or_else(|| "atp_api=info,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env()?;
    let db = Db::connect(config.database_url.as_deref())?;
    let state = AppState::new(db).with_auth(JwtVerifier::from_env());
    let app = build_app(&config, state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port)).await?;
    tracing::info!("api listening on http://localhost:{}", config.port);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;
    Ok(())
}
