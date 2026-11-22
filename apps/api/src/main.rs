use atp_api::{AppState, build_app, config::Config, db::Db};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // .env chỉ dùng khi chạy local; trên Render biến môi trường đặt trong dashboard.
    dotenvy::dotenv().ok();
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "atp_api=info,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env()?;
    let db = Db::connect(config.database_url.as_deref())?;
    let app = build_app(&config, AppState::new(db));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port)).await?;
    tracing::info!("api listening on http://localhost:{}", config.port);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;
    Ok(())
}
