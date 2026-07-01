use axum::{routing::get, Router};

use super::server::ws_handler;
use crate::state::WS_STATE;

pub fn router() -> Router {
    Router::new()
        .route("/ws", get(ws_handler))
        .with_state(WS_STATE.get().expect("WS_STATE not initialized").clone())
}
