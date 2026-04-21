use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use tokio::net::TcpListener;
use web_server::{config::Config, migration::Migrator, routes};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let config = Config::from_env();

    std::fs::create_dir_all("data")?;
    let db = Database::connect(&config.database_url).await?;
    Migrator::up(&db, None).await?;

    println!("Database connected: {}", config.database_url);

    let app = routes::routes(db);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
    println!("Server running on http://localhost:{}", config.port);
    axum::serve(listener, app).await?;

    Ok(())
}
