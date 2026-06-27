use super::handlers;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

pub fn router() -> Router {
    Router::new()
        .route("/ping", get(handlers::ping))
        .route("/message", post(handlers::message))
        .route("/open_tray_page", post(handlers::open_tray_page))
        .layer(CorsLayer::permissive())
}
