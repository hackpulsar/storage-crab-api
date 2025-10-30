use actix_web::{post, web, HttpRequest, HttpResponse};
use redis::AsyncCommands;
use serde::Deserialize;
use sqlx::Row;

use crate::AppState;
use crate::models::jwt::{JwtTokenPair, TokenType};
use crate::models::user::{DBUser, UserInfo, UserLoginCredentials};
use crate::services::auth::{get_and_validate_jwt, validate_jwt};
use crate::utils::errors::AppError;

#[post("/api/users/greet/")]
pub async fn greet(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let token = get_and_validate_jwt(&req, &data.secret)?;
    let user_id = token.claims.sub.parse::<i32>().map_err(|_| AppError::InternalServerError { 
        msg: "Failed to parse sub to id".to_string() 
    })?;

    // Look up username in a database
    let record = sqlx::query("select username from users where id = $1")
        .bind(user_id)
        .fetch_one(&data.db_pool)
        .await
        // TODO: do not propagate SQL error
        .map_err(|_| AppError::InternalServerError { msg: "Query failed".to_string() })?;

    Ok(HttpResponse::Ok().body(format!("Welcome back, {}", record.get::<String, _>("username"))))
}

#[post("/api/users/")]
pub async fn create_user(user: web::Json<DBUser>, data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    // First, look for a record with given email in the DB
    let record = sqlx::query("select 1 from users where email = $1")
        .bind(user.email.clone())
        .fetch_optional(&data.db_pool)
        .await
        .map_err(|_| AppError::InternalServerError { msg: "Failed to fetch user".to_string() })?;

    if let Some(_) = record {
        return Err(AppError::BadRequest { msg: "User with this email already exists".to_string() });
    }

    // User JSON to User struct
    // TODO: do we need the entire user ESPECIALLY the password hash?
    // NOT SAFE
    let user: DBUser = user.into_inner();

    // Perform a query
    let record = sqlx::query("insert into users(email, username, password) values ($1, $2, $3) returning id, email, username")
        .bind(user.email.clone())
        .bind(user.username.clone())
        .bind(user.password_hash.clone())
        .fetch_one(&data.db_pool)
        .await
        // TODO: do not propagate SQL error
        .map_err(|_| AppError::InternalServerError { msg: "Insert user query failed".to_string() })?;

    Ok(HttpResponse::Ok().json(UserInfo {
        id: record.get("id"),
        email: record.get("email"),
        username: record.get("username")
    }))
}

#[post("api/token/get/")]
async fn login(user: web::Json<UserLoginCredentials>, data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    // Look up user with given email
    let record = sqlx::query("select password, id from users where email = $1")
        .bind(user.email.clone())
        .fetch_optional(&data.db_pool)
        .await
        .map_err(|_| AppError::InternalServerError { msg: "Login query failed".to_string() })?;

    // Send jwt token pair on successful login
    match record {
        Some(record) => {
            if user.verify_password(&record.get::<String, _>("password")) {
                Ok(HttpResponse::Ok().json(JwtTokenPair::generate_for(
                    record.get::<i32, _>("id").to_string(),
                    data.secret.clone()
                )))
            } else {
                Err(AppError::BadRequest { msg: "Wrong password".to_string() })
            }
        },
        None => Err(AppError::BadRequest { msg: "No user found with given credentials".to_string() }),
    }
}

// Represents a refresh token request body
#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[post("/api/token/refresh/")]
async fn refresh_token(req: web::Json<RefreshRequest>, data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let token = validate_jwt(req.refresh_token.clone().as_str(), data.secret.clone().as_str())
        .ok_or(AppError::BadRequest { msg: "Invalid or expired refresh token".to_string() })?;

    if token.claims.token_type != TokenType::Refresh {
        return Err(AppError::BadRequest { msg: "Wrong token type".to_string() });
    }

    // Check if token is blacklisted
    let mut conn = data.redis_pool
        .get().await
        .map_err(|_| AppError::InternalServerError { msg: "Connection to Redis lost".to_string() })?;

    // If token exists in Redis, it is blacklisted
    match conn.get::<_, Option<String>>(token.claims.jti.clone()).await.ok() {
        Some(_) => Err(AppError::BadRequest { msg: "Token is blacklisted".to_string() }),
        None => {
            // Blacklist token. 
            // Redis will delete this entry as soon as the token gets expired.
            conn.set_ex::<_, _, ()>(
                token.claims.jti.clone(),
                req.refresh_token.clone(),
                // saturating_sub wraps to zero to prevent underflow
                token.claims.exp.saturating_sub(chrono::Utc::now().timestamp() as usize) as u64
            ).await
            .map_err(|_| { AppError::InternalServerError { msg: "Failed to blacklist the token".to_string() } })?;

            // Send back refreshed token pair
            Ok(HttpResponse::Ok().json(JwtTokenPair::generate_for(
                token.claims.sub,
                data.secret.clone()
            )))
        }
    }
}