use axum::{Router, routing::{post, get}};
use tower_http::services::ServeDir;

use crate::{handlers, models::WarpLinkState};

pub fn create_app() -> Router<WarpLinkState> {

    Router::new()
    .route("/", get(handlers::pages::index))
    .route("/register", post(handlers::links::register_short_link))
    .route("/:short_link", get(handlers::links::redirect_to_long_link))
    .route("/health", get(handlers::health::health_check))
    .nest_service("/static", ServeDir::new("static"))
  
}