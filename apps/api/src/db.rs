use std::time::Duration;

use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};

/// Trạng thái kết nối database, trả về trong `/health`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DbStatus {
    Connected,
    NotConfigured,
    Error,
}

/// Pool kết nối Postgres (Supabase) dùng chung cho cả app. Clone rẻ: bên trong là Arc.
#[derive(Clone)]
pub struct Db {
    pool: Option<PgPool>,
}

impl Db {
    /// Không có database: api vẫn chạy, `/health` báo `not_configured`. Dùng cho test.
    pub fn disabled() -> Self {
        Self { pool: None }
    }

    /// Tạo pool từ `DATABASE_URL`. Pool kết nối lười: api khởi động được cả khi
    /// database tạm thời không tới được, lỗi chỉ hiện ở `/health` và ở request dùng DB.
    /// URL sai định dạng thì trả lỗi ngay để api không chạy với cấu hình hỏng.
    pub fn connect(database_url: Option<&str>) -> anyhow::Result<Self> {
        let Some(url) = database_url else {
            tracing::warn!("DATABASE_URL chưa đặt — xem apps/api/.env.example");
            return Ok(Self::disabled());
        };
        let pool = PgPoolOptions::new()
            // Gói Free của Supabase giới hạn số kết nối, giữ pool nhỏ.
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect_lazy(url)
            .map_err(|err| anyhow::anyhow!("DATABASE_URL không hợp lệ: {err}"))?;
        Ok(Self { pool: Some(pool) })
    }

    /// Lấy pool để truy vấn trong handler; `None` nếu chưa cấu hình.
    pub fn pool(&self) -> Option<&PgPool> {
        self.pool.as_ref()
    }

    pub async fn ping(&self) -> DbStatus {
        let Some(pool) = &self.pool else {
            return DbStatus::NotConfigured;
        };
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => DbStatus::Connected,
            Err(err) => {
                tracing::error!(%err, "ping database lỗi");
                DbStatus::Error
            }
        }
    }
}
