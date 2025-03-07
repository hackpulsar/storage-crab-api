use actix_web::{post, web, HttpRequest, HttpResponse};
use redis::AsyncCommands;
use serde::Deserialize;
use sqlx::Row;
use crate::AppState;
use crate::models::jwt::{JwtTokenPair, TokenType};
use crate::models::user::{RegisterUser, RegisterUserResponse, UserLoginCredentials};
use crate::services::auth::{get_jwt_from, validate_jwt};
use crate::utils::errors::AppError;

#[post("/api/users/greet/")]
pub async fn greet(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let token = get_jwt_from(&req)?;
    match validate_jwt(token, data.secret.clone().as_str()) {
        Some(decoded_data) => {
            // Look up username in a database
            let query = "select username from users where email = $1";
            let res = sqlx::query(query)
                .bind(decoded_data.claims.sub)
                .fetch_one(&data.db)
                .await;

            match res {
                Ok(row) => Ok(HttpResponse::Ok()
                    .body(format!("Welcome back, {}", row.get::<String, _>("username")))),
                Err(_) => Err(AppError::InternalServerError { msg: "Query failed".to_string() }),
            }
        },
        None => Err(AppError::Unauthorized)
    }
}

// Creates a new user in a database
#[post("/api/users/")]
pub async fn create_user(
    user: web::Json<RegisterUser>,
    data: web::Data<AppState>
) -> Result<HttpResponse, AppError> {
    // User JSON to User struct
    let user = user.into_inner();

    // Perform a query
    let query = "insert into users(email, username, password) values ($1, $2, $3) returning id";
    let res = sqlx::query(query)
        .bind(user.email.clone())
        .bind(user.username.clone())
        .bind(user.password_hash.clone())
        .fetch_one(&data.db)
        .await;

    // If success, send back a JSON with the created user credentials and an ID.
    // Otherwise, send back an internal server error.
    match res {
        Ok(record) => Ok(HttpResponse::Ok().json(RegisterUserResponse {
            id: record.get("id"),
            user
        })),
        Err(_) => Err(AppError::InternalServerError { msg: "Insert user query failed".to_string() }),
    }
}

// Token pair obtain endpoint
#[post("api/token/get/")]
async fn login(
    user: web::Json<UserLoginCredentials>,
    data: web::Data<AppState>
) -> Result<HttpResponse, AppError> {
    // Look up user with given email
    let query = "select password from users where email = $1";
    let res = sqlx::query(query)
        .bind(user.email.clone())
        .fetch_optional(&data.db)
        .await;

    // Send jwt token pair on successful login
    let row = res.map_err(|_| AppError::InternalServerError { msg: "Login query failed".to_string() })?;
    match row {
        Some(entry) => {
            if user.verify_password(&entry.get::<String, _>("password")) {
                let user_email: String = user.email.clone();
                Ok(HttpResponse::Ok().json(JwtTokenPair::generate_for(
                    user_email,
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

// Token refresh endpoint
#[post("/api/token/refresh/")]
async fn refresh_token(
    req: web::Json<RefreshRequest>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    match validate_jwt(req.refresh_token.clone().as_str(), data.secret.clone().as_str()) {
        Some(decoded_data) => {
            if decoded_data.claims.token_type != TokenType::Refresh {
                return Err(AppError::BadRequest { msg: "Wrong token type".to_string() });
            }

            // Check if token is blacklisted
            let mut conn = data.redis_pool.get().await
                .map_err(|_| AppError::InternalServerError {msg: "Connection to Redis failed".to_string() })?;
            let is_blacklisted: Option<String> = conn.get(decoded_data.claims.jti.clone()).await.ok();
            match is_blacklisted {
                Some(_) => Err(AppError::BadRequest { msg: "Token is blacklisted".to_string() }),
                None => {
                    // Blacklist refresh token used with expiration date.
                    // Redis will delete this entry as soon as the token gets expired.
                    let _: () = conn.set_ex(
                        decoded_data.claims.jti.clone(),
                        req.refresh_token.clone(),
                        (decoded_data.claims.exp - (chrono::Utc::now().timestamp() as usize)) as u64
                    ).await.unwrap();

                    // Refresh token pair
                    Ok(HttpResponse::Ok().json(JwtTokenPair::generate_for(
                        decoded_data.claims.sub,
                        data.secret.clone()
                    )))
                }
            }
        },
        None => Err(AppError::BadRequest { msg: "Invalid or expired refresh token".to_string() })
    }
}