use axum::{Router, routing::{post, get}};

use crate::{handlers, models::WarpLinkState};

pub fn create_app() -> Router<WarpLinkState> {

    Router::new()
    .route("/register", post(handlers::links::register_short_link))
    .route("/:short_link", get(handlers::links::redirect_to_long_link))
    .route("/health", get(handlers::health::health_check))
  
}