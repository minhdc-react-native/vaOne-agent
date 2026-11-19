pub mod handlers;
pub mod routes;
pub mod types;
use axum::serve;
use tokio::net::TcpListener;

pub async fn start() -> Result<(), String> {
    let start = std::time::Instant::now();

    println!("[SERVER] 1. Start");

    let app = routes::router();

    println!("[SERVER] 2. Router created: {:?}", start.elapsed());

    let listener = TcpListener::bind("127.0.0.1:15682")
        .await
        .map_err(|e| format!("Cannot bind local server: {e}"))?;

    println!("[SERVER] 3. Listener bound: {:?}", start.elapsed());

    println!("Server started: http://127.0.0.1:15682");

    serve(listener, app)
        .await
        .map_err(|e| format!("Local server error: {e}"))?;

    Ok(())
}
