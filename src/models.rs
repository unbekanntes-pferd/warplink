use axum::{response::IntoResponse, Json, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
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
    ServerError
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
pub struct CreateWarpLinkRequest {
    pub long_link: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct WarpLinkErrorResponse {
    pub message: String,
    pub status: u32,
    pub details: Option<String>
}

impl WarpLinkErrorResponse {
    fn new(status: u32, message: impl Into<String>, details: Option<String>) -> Self {
        Self {
            message: message.into(),
            status,
            details,
        }
    }

    pub fn new_error(status: u16, message: impl Into<String>, details: Option<String>) -> impl IntoResponse {

        let status_code: StatusCode = status.try_into().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let response = WarpLinkErrorResponse::new(status.into(), message, details);

        (status_code, Json(response))

    }

    pub fn new_internal_error(details: Option<String>) -> impl IntoResponse {
        let message = "Internal server error.";
        Self::new_error(500, message, details)
    }

    pub fn new_not_found(details: Option<String>) -> impl IntoResponse {
        let message = "Not found.";
        Self::new_error(404, message, details)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WarpLinkConfig {
    port: u32,
    database_url: String,
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

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
            let database_user = std::env::var("DB_USER").unwrap_or("postgres".to_string());

            let database_password =
            std::env::var("DB_PASSWORD").unwrap_or("postgres".to_string());
            format!(
                "postgres://{}:{}@warplink-db:5432/warplink",
                database_user, database_password
            )
            });

        Self { port, database_url }
    }

    pub fn port(&self) -> u32 {
        self.port
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }
}