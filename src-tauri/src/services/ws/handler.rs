use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

use crate::state::WsState;

pub async fn handle_socket(socket: WebSocket, state: WsState) {
    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    state.add_client(tx);

    // Task gửi dữ liệu từ server -> client
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Task nhận dữ liệu từ client (nếu sau này cần)
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    println!("Receive: {}", text);
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    let _ = tokio::join!(send_task, recv_task);

    println!("WebSocket disconnected");
}
