pub mod handlers;
pub mod routes;
pub mod types;
use axum::serve;
use tokio::net::TcpListener;

pub async fn start() {
    let app = routes::router();

    let listener = TcpListener::bind("127.0.0.1:15682").await.unwrap();

    println!("Server started: http://127.0.0.1:15682");

    serve(listener, app).await.unwrap();
}
