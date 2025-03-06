use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use jsonwebtoken::{decode, DecodingKey, Validation};
use redis::AsyncCommands;
use sqlx::{Row};
use serde::{Deserialize, Serialize};
use crate::jwt_token_pair::{Claims, JwtTokenPair, TokenType};
use crate::AppState;

// Represents a user
#[derive(Serialize, Deserialize)]
pub struct User {
    email: String,
    username: String,
    password: String,
}

// Response to a create user request
#[derive(Serialize)]
struct CreateUserResponse {
    id: i32,
    user: User,
}

// User credentials on login
#[derive(Deserialize)]
struct UserLoginCredentials {
    email: String,
    password: String,
}

impl UserLoginCredentials {
    // Compares given password to user password
    pub fn verify_password(&self, password: &str) -> bool {
        self.password == password
    }
}

#[post("/api/users/greet/")]
pub async fn greet(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let auth_header = req.headers().get("Authorization").unwrap().to_str().unwrap();
    let token = auth_header.strip_prefix("Bearer ").unwrap();

    // Validate
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(data.secret.clone().as_bytes()),
        &Validation::default()
    ).ok();

    match decoded {
        Some(decoded_data) => {
            // Look up username in a database
            let query = "select username from users where email = $1";
            let res = sqlx::query(query)
                .bind(decoded_data.claims.sub)
                .fetch_one(&data.db)
                .await;

            match res {
                Ok(row) => HttpResponse::Ok().body(format!("Welcome back, {}", row.get::<String, _>("username"))),
                Err(_) => HttpResponse::InternalServerError().body("Something went wrong"),
            }

        },
        None => HttpResponse::Unauthorized().body("Unauthorized access")
    }
}

// Creates a new user in a database
#[post("/api/users/")]
pub async fn create_user(
    user: web::Json<User>,
    data: web::Data<AppState>
) -> impl Responder {
    // User JSON to User struct
    let user = user.into_inner();

    // Perform a query
    let query = "insert into users(email, username, password) values ($1, $2, $3) returning id";
    let res = sqlx::query(query)
        .bind(user.email.clone())
        .bind(user.username.clone())
        .bind(user.password.clone())
        .fetch_one(&data.db)
        .await;

    // If success, send back a JSON with the created user credentials and an ID.
    // Otherwise, send back an internal server error.
    match res {
        Ok(record) => HttpResponse::Ok().json(CreateUserResponse {
            id: record.get("id"),
            user
        }),
        Err(_) => HttpResponse::InternalServerError().body("Error while inserting user")
    }
}

// Token pair obtain endpoint
#[post("api/token/get/")]
async fn login(
    user: web::Json<UserLoginCredentials>,
    data: web::Data<AppState>
) -> impl Responder {
    // Look up user with given email
    let query = "select password from users where email = $1";
    let res = sqlx::query(query)
        .bind(user.email.clone())
        .fetch_optional(&data.db)
        .await;

    // Send jwt token pair on successful login
    match res {
        Ok(row) => {
            match row {
                Some(entry) => {
                    if user.verify_password(&entry.get::<String, _>("password")) {
                        let user_email: String = user.email.clone();
                        HttpResponse::Ok().json(JwtTokenPair::generate_for(
                            user_email,
                            data.secret.clone()
                        ))
                    } else {
                        HttpResponse::NotFound().body("Wrong password")
                    }
                },
                None => HttpResponse::NotFound().body("No user found with given credentials")
            }
        },
        Err(_) => HttpResponse::InternalServerError().body("Error while logging in")
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
) -> impl Responder {
    // Validate
    let decoded = decode::<Claims>(
        req.refresh_token.as_str(),
        &DecodingKey::from_secret(data.secret.as_bytes()),
        &Validation::default()
    ).ok();

    match decoded {
        Some(decoded_data) => {
            if decoded_data.claims.token_type != TokenType::Refresh {
                return HttpResponse::Unauthorized().body("Wrong token type");
            }

            // Check if token is blacklisted
            match data.redis_pool.get().await {
                Ok(mut conn) => {
                    let is_blacklisted: Option<String> = conn.get(decoded_data.claims.jti.clone()).await.ok();
                    match is_blacklisted {
                        Some(_) => HttpResponse::Unauthorized().body("Token is blacklisted."),
                        None => {
                            // Blacklist refresh token used with expiration date.
                            // Redis will delete this entry as soon as the token gets expired.
                            let _: () = conn.set_ex(
                                decoded_data.claims.jti.clone(),
                                req.refresh_token.clone(),
                                (decoded_data.claims.exp - (chrono::Utc::now().timestamp() as usize)) as u64
                            ).await.unwrap();

                            // Refresh token pair
                            HttpResponse::Ok().json(JwtTokenPair::generate_for(
                                decoded_data.claims.sub,
                                data.secret.clone()
                            ))
                        }
                    }
                },
                Err(_) => HttpResponse::InternalServerError().body("Connection to Redis failed.")
            }
        },
        None => HttpResponse::Unauthorized().body("Invalid or expired refresh token")
    }
}
