mod models;
mod routes;
mod services;
mod utils;

use core::panic;

use crate::routes::init_routes;
use crate::utils::generate_shared_secret;
use actix_web::{web, App, HttpServer};
use deadpool_redis::{Config, Runtime};
use sqlx::{Postgres};

// Holds app state
pub struct AppState {
    secret: String,
    db_pool: sqlx::Pool<Postgres>,
    redis_pool: deadpool_redis::Pool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Loading environment variables
    dotenv::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    println!("Connecting to the database...");
    let db_pool = match create_db_pool(db_url).await {
        Ok(pool) => pool,
        Err(e) => panic!("DB connection failed: {}", e)
    };
    println!("Successfully connected to the database.");

    println!("Connecting to Redis...");
    let redis_pool = match create_redis_pool(redis_url) {
        Ok(pool) => pool,
        Err(e) => panic!("Failed to create Redis Pool: {}", e)
    };
    println!("Connected to Redis.");

    println!("Running database migrations...");
    sqlx::migrate!().run(&db_pool).await.expect("Failed to run database migrate.");
    println!("Migrations successful.");

    let secret = generate_shared_secret();

    // Starting a web server
    println!("Starting server.");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                secret: secret.clone(),
                db_pool: db_pool.clone(),
                redis_pool: redis_pool.clone(),
            }))
            .configure(init_routes)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

// Connects to a database
async fn create_db_pool(db_url: String) -> Result<sqlx::Pool<Postgres>, sqlx::Error> {
    let pool = sqlx::postgres::PgPool::connect(db_url.as_str()).await?;
    Ok(pool)
}

// Creates a new redis pool
fn create_redis_pool(redis_url: String) -> Result<deadpool_redis::Pool, deadpool_redis::CreatePoolError> {
    let pool = Config::from_url(redis_url.as_str()).create_pool(Some(Runtime::Tokio1))?;
    Ok(pool)
}
