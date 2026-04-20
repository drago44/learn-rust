use tokio::net::TcpListener;
use web_server::{config::Config, repositories, routes};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let config = Config::from_env();
    let pool = repositories::create_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    println!("Database connected: {}", config.database_url);

    let app = routes::routes(pool);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();
    println!("Server running on http://localhost:{}", config.port);
    axum::serve(listener, app).await.unwrap();
}
