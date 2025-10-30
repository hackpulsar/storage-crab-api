use actix_web::{get, web, HttpRequest, HttpResponse};

use crate::models::user::UserInfo;
use crate::services::auth::get_and_validate_jwt;
use crate::utils::errors::AppError;
use crate::AppState;
use sqlx::Row;

// Fetches information about user.
// Email and username.
#[get("/api/users/me/")]
pub async fn me(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let token = get_and_validate_jwt(&req, &data.secret)?;
    let user_id = token.claims.sub.parse::<i32>().map_err(|_| AppError::InternalServerError { 
        msg: "Failed to parse sub to id".to_string() 
    })?;

    // Retrieveing information about user
    let record = sqlx::query("select * from users where id = $1")
        .bind(user_id)
        .fetch_one(&data.db_pool)
        .await
        // TODO: do not propagte the SQL error. Instead, log it to a file.
        .map_err(|_| AppError::InternalServerError { msg: "Query failed".to_string() })?;

    Ok(HttpResponse::Ok().json(UserInfo {
        id: user_id,
        email: record.get("email"),
        username: record.get("username")
    }))
}