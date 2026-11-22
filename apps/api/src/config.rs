use std::env;

/// Cấu hình đọc từ biến môi trường (và file `.env` khi chạy local).
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    /// Origin của web được gọi api, nhiều giá trị cách nhau bằng dấu phẩy.
    pub web_origins: Vec<String>,
    /// Chuỗi kết nối Postgres của Supabase.
    /// Để trống thì api vẫn chạy, `/health` báo `db = not_configured`.
    pub database_url: Option<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let port = match env::var("PORT") {
            Ok(v) => v
                .parse()
                .map_err(|_| anyhow::anyhow!("PORT không hợp lệ: {v}"))?,
            Err(_) => 4000,
        };
        let web_origins = env::var("WEB_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:3000".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let database_url = env::var("DATABASE_URL").ok().filter(|s| !s.is_empty());

        Ok(Self {
            port,
            web_origins,
            database_url,
        })
    }
}
