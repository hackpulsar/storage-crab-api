use actix_web::{get, web, HttpRequest, HttpResponse};

use crate::models::user::UserInfoReponse;
use crate::services::auth::get_and_validate_jwt;
use crate::utils::errors::AppError;
use crate::AppState;
use sqlx::Row;

#[get("/api/users/me/")]
pub async fn me(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let token = get_and_validate_jwt(&req, data.secret.clone().as_str())?;

    let res: Result<sqlx::postgres::PgRow, sqlx::Error> = sqlx::query("select * from users where id = $1")
        .bind(
            token.claims.sub
                .parse::<i32>()
                .map_err(|_| AppError::InternalServerError { msg: "Failed to parse sub to id".to_string() })?
        )
        .fetch_one(&data.db)
        .await;

    match res {
        Ok(record) => Ok(
            HttpResponse::Ok().json(UserInfoReponse {
                email: record.get("email"),
                username: record.get("username")
            })
        ),
        Err(_) => Err(AppError::InternalServerError { msg: "Query failed".to_string() }),
    }
}