use actix_web::{get, web, HttpRequest, HttpResponse};

use crate::{models::user::UserInfo, routes::user};
use crate::services::auth::get_and_validate_jwt;
use crate::utils::errors::AppError;
use crate::AppState;
use sqlx::Row;
use log::{warn, debug};

// Fetches information about user.
// Email and username.
#[get("/api/users/me/")]
pub async fn me(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let token = get_and_validate_jwt(&req, &data.secret)?;
    let user_id = token.claims.sub.parse::<i32>().map_err(|_| { 
        warn!("Failed to parse sub to id [{:?}]", token.claims);
        AppError::InternalServerError { msg: "Failed to parse sub to id".to_string() }
    })?;

    // Retrieveing information about user
    let record = sqlx::query("select * from users where id = $1")
        .bind(user_id)
        .fetch_one(&data.db_pool)
        .await
        .map_err(|e| {
            warn!("User fetch query failed: {:?}", e);
            AppError::InternalServerError { msg: "User fetch query failed".to_string() }
        })?;

    let email = record.get("email");
    let username = record.get("username");

    debug!("Fetched user data [user_id: {}, email: {}, username: {} ]", user_id, email, username);

    Ok(HttpResponse::Ok().json(UserInfo {
        id: user_id,
        email: email,
        username: username
    }))
}