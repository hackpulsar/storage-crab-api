mod auth_services;
mod jwt_token_pair;

use actix_web::{web, App, HttpServer};
use deadpool_redis::{Config, Runtime};
use sqlx::{Pool, Postgres};
use crate::jwt_token_pair::generate_shared_secret;

// Holds app state
pub struct AppState {
    secret: String,
    db: Pool<Postgres>,
    redis_pool: deadpool_redis::Pool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Loading environment variables
    dotenv::dotenv().ok();

    // Database URL
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    // Connecting to a database
    println!("Connecting to the database...");
    let pool = sqlx::postgres::PgPool::connect(db_url.as_str())
        .await
        .expect("DB connection failed");
    println!("Successfully connected to the database.");

    println!("Connecting to Redis...");
    let redis_pool = create_redis_pool(redis_url).await;
    println!("Connected to Redis.");

    // Run migrations
    println!("Running database migrations...");
    match sqlx::migrate!().run(&pool).await {
        Ok(_) => (),
        Err(_) => panic!("Failed to run database migrate.")
    }
    println!("Migrations successful.");

    // Generating shared secret
    let secret = generate_shared_secret();

    // Starting a web server
    println!("Starting server.");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                secret: secret.clone(),
                db: pool.clone(),
                redis_pool: redis_pool.clone(),
            }))
            .service(auth_services::create_user)
            .service(auth_services::greet)
            .service(auth_services::login)
            .service(auth_services::refresh_token)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// Creates new redis pool from URL
async fn create_redis_pool(redis_url: String) -> deadpool_redis::Pool {
    match Config::from_url(redis_url.as_str()).create_pool(Some(Runtime::Tokio1)) {
        Ok(pool) => pool,
        Err(e) => panic!("Failed to create Redis Pool: {}", e),
    }
}
