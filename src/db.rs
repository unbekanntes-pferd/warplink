use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, debug};

use crate::models::WarpLinkError;

pub async fn create_connection_pool(db_uri: &str, max_conn: Option<u32>) -> Result<PgPool, WarpLinkError> {

    let max_conn = max_conn.unwrap_or(20);

    PgPoolOptions::new()
        .max_connections(max_conn)
        .connect(db_uri)
        .await
        .map_err(|err| {
            error!("Failed to connect to Postgres DB: {}", err);
            debug!("Failed to connect to DB uri: {}", db_uri);
            WarpLinkError::DatabaseError(err.to_string())
        })
}