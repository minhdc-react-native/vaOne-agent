use super::handlers;
use crate::services::ws;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

pub fn router() -> Router {
    Router::new()
        .route("/ping", get(handlers::ping))
        .route("/sync_token", post(handlers::sync_token))
        .route("/message", post(handlers::message))
        .route("/open_tray_page", post(handlers::open_tray_page))
        .merge(ws::router::router())
        .layer(CorsLayer::permissive())
}
