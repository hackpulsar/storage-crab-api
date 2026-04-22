pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

use deadpool_redis::{Config, Runtime};
use sqlx::{Postgres};
use log::info;

// Holds app state
pub struct AppState {
    pub secret: String,
    pub db_pool: sqlx::Pool<Postgres>,
    pub redis_pool: deadpool_redis::Pool,
}

// Connects to a database
pub async fn create_db_pool(db_url: String) -> Result<sqlx::Pool<Postgres>, sqlx::Error> {
    info!("Connecting to PostgreSQL...");
    let pool = sqlx::postgres::PgPool::connect(db_url.as_str()).await?;
    info!("Connected to PostgreSQL.");
    Ok(pool)
}

// Creates a new redis pool
pub fn create_redis_pool(redis_url: String) -> Result<deadpool_redis::Pool, deadpool_redis::CreatePoolError> {
    info!("Connecting to Redis...");
    let pool = Config::from_url(redis_url.as_str()).create_pool(Some(Runtime::Tokio1))?;
    info!("Connected to Redis.");
    Ok(pool)
}