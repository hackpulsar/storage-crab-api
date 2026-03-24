mod models;
mod routes;
mod services;
mod utils;

use core::panic;

use actix_multipart::form::tempfile::TempFileConfig;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix_web::{App, HttpServer, web};
use deadpool_redis::{Config, Runtime};
use sqlx::{Postgres};
use log::{info, error};

use crate::routes::init_routes;
use crate::utils::generate_shared_secret;

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

    env_logger::init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let db_pool = match create_db_pool(db_url).await {
        Ok(pool) => pool,
        Err(e) => panic!("DB connection failed: {}", e)
    };

    let redis_pool = match create_redis_pool(redis_url) {
        Ok(pool) => pool,
        Err(e) => panic!("Failed to create Redis Pool: {}", e)
    };

    info!("Running database migrations...");
    sqlx::migrate!().run(&db_pool).await.expect("Failed to run database migrate.");
    info!("Migrations successful.");

    let secret = generate_shared_secret();
    info!("Shared secret generated.");

    info!("Setting up certificates...");
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .map_err(|e| {
            error!("Setting private key failed with error: {:?}", e);
            e
        })?;
    builder.set_certificate_chain_file("cert.pem")
        .map_err(|e| {
            error!("Setting certificate chain file failed with error: {:?}", e);
            e
        })?;
    info!("Certificates set.");

    // Starting a web server
    info!("Starting server.");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                secret: secret.clone(),
                db_pool: db_pool.clone(),
                redis_pool: redis_pool.clone(),
            }))
            .app_data(TempFileConfig::default().directory(std::env::var("FILES_STORAGE_PATH").unwrap()))
            .configure(init_routes)
    })
    .bind_openssl("0.0.0.0:8080", builder)?
    .run()
    .await
}

// Connects to a database
async fn create_db_pool(db_url: String) -> Result<sqlx::Pool<Postgres>, sqlx::Error> {
    info!("Connecting to PostgreSQL...");
    let pool = sqlx::postgres::PgPool::connect(db_url.as_str()).await?;
    info!("Connected to PostgreSQL.");
    Ok(pool)
}

// Creates a new redis pool
fn create_redis_pool(redis_url: String) -> Result<deadpool_redis::Pool, deadpool_redis::CreatePoolError> {
    info!("Connecting to Redis...");
    let pool = Config::from_url(redis_url.as_str()).create_pool(Some(Runtime::Tokio1))?;
    info!("Connected to Redis.");
    Ok(pool)
}
