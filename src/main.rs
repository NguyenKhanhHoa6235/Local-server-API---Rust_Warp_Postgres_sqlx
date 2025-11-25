
mod db;
mod handlers;
mod models;
mod routes;
mod errors;
mod jwt;
mod rate_limit;

use sqlx::PgPool;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;

    // Chạy migration
    sqlx::migrate!("./migrations").run(&pool).await.expect("Migration failed");

    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".into());
    let bind_port: u16 = std::env::var("BIND_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3030);

    // Tạo routes từ module routes
    let routes = routes::create_routes(pool);

    println!("Server running on http://{}:{}", bind_address, bind_port);

    warp::serve(routes)
        .run((bind_address.parse::<std::net::IpAddr>()?, bind_port))
        .await;

    Ok(())
}
