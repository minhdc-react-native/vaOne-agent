use axum::{extract::ws::WebSocketUpgrade, extract::State, response::Response};

use crate::services::ws::handler::handle_socket;
use crate::state::WsState;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<WsState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}
