use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, PartialEq, Eq, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WarpLink {
    pub id: i64,
    pub short_link: String,
    pub long_link: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
pub enum WarpLinkError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Server error - cannot run server.")]
    ServerError,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWarpLinkRequest {
    pub long_link: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct WarpLinkErrorResponse {
    pub message: String,
    pub status: u32,
    pub details: Option<String>,
}

impl WarpLinkErrorResponse {
    fn new(status: u32, message: impl Into<String>, details: Option<String>) -> Self {
        Self {
            message: message.into(),
            status,
            details,
        }
    }

    pub fn new_internal_error(details: Option<String>) -> Self {
        let message = "Internal server error.";
        Self::new(500, message, details)
    }

    pub fn new_not_found(details: Option<String>) -> Self {
        let message = "Not found.";
        Self::new(404, message, details)
    }

    pub fn new_bad_request(details: Option<String>) -> Self {
        let message = "Bad request.";
        Self::new(400, message, details)
    }
}

impl IntoResponse for WarpLinkErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let status_code: StatusCode =
            StatusCode::from_u16(self.status as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status_code, Json(self)).into_response()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WarpLinkConfig {
    port: u32,
    database_url: String,
    is_prod: bool,
}

#[derive(Debug, Clone)]
pub struct WarpLinkState {
    db_pool: sqlx::PgPool,
}

impl WarpLinkState {
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        Self { db_pool }
    }

    pub fn pool(&self) -> &sqlx::PgPool {
        &self.db_pool
    }
}

impl WarpLinkConfig {
    pub fn new() -> Self {
        let port: u32 = std::env::var("PORT")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(3000);

        let is_prod = std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "dev".to_string())
            .eq("prod");

        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            let database_user = std::env::var("DB_USER").unwrap_or("postgres".to_string());

            let database_password = std::env::var("DB_PASSWORD").unwrap_or("postgres".to_string());
            format!(
                "postgres://{}:{}@warplink-db:5432/warplink",
                database_user, database_password
            )
        });

        let database_url = if is_prod {
            format!("{}?sslmode=require", database_url)
        } else {
            database_url
        };

        Self {
            port,
            database_url,
            is_prod,
        }
    }

    pub fn port(&self) -> u32 {
        self.port
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }
}
