use tokio::net::TcpListener;

mod adapters;
mod domain;
mod ports;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, axum::Router::new()).await.unwrap();
}
