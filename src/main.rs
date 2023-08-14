use models::{WarpLinkError, WarpLinkConfig, WarpLinkState};
use tracing::{error, debug};

mod handlers;
mod routes;
mod models;
mod db;

#[tokio::main]
async fn main() -> Result<(), WarpLinkError> {
        
        // init tracing
        tracing_subscriber::fmt::init();

        // setup config
        let config = WarpLinkConfig::new();
        let db_pool = db::create_connection_pool(config.database_url(), None).await?;
        let app_state = WarpLinkState::new(db_pool);

        // run migrations
        sqlx::migrate!()
            .run(app_state.pool())
            .await
            .map_err(|err| {
                error!("Failed to run migrations: {}", err);
                WarpLinkError::DatabaseError(err.to_string())
            })?;

        // setup app
        let app = routes::create_app().with_state(app_state);

        let app_url = format!("0.0.0.0:{}", config.port());

        // run app
        axum::Server::bind(&app_url.parse().expect("Invalid app address"))
            .serve(app.into_make_service())
            .await
            .map_err(|err| {
                error!("Failed to run on port {}", config.port());
                debug!("Error: {}", err);
                WarpLinkError::ServerError
            })?;

        Ok(())
}
