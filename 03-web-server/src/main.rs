use tokio::net::TcpListener;

mod adapters;
mod domain;
mod ports;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = adapters::db::create_pool(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("Database connected: {}", database_url);

    let app = adapters::routes::routes(pool);
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
