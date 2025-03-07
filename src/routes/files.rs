use actix_multipart::form::{MultipartForm};
use actix_web::{post, web, HttpRequest, HttpResponse};
use std::fs;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use sqlx::Row;

use crate::AppState;
use crate::models::file::*;
use crate::services::auth::{get_jwt_from, validate_jwt};
use crate::utils::errors::AppError;

// Searches for all the files associated with given user
/*#[post("/api/files/")]
async fn get_files(
    data: web::Data<AppState>,
) -> impl Responder {

}*/

// Uploads a file to an authorized user storage
#[post("/api/files/upload/")]
async fn upload_file(
    req: HttpRequest,
    MultipartForm(form): MultipartForm<UploadForm>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    // Validate token
    let token = get_jwt_from(&req)?;
    match validate_jwt(token, data.secret.clone().as_str()) {
        Some(decoded_data) => {
            // Retrieve username from db
            let query = "select username from users where email = $1";
            let res = sqlx::query(query)
                .bind(decoded_data.claims.sub)
                .fetch_one(&data.db)
                .await;

            let row = res
                .map_err(|_| AppError::InternalServerError { msg: "Username retrieve query failed".to_string() } )?;

            let username: String = row.get("username");
            // Hash username
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let username_hash = argon2.hash_password(username.as_bytes(), &salt)
                .map_err(|_| AppError::InternalServerError { msg: "Username hash failed".to_string() } )?
                .to_string();

            // Path to files storage
            let storage_path = std::env::var("FILES_STORAGE_PATH").unwrap();

            // Create dirs for file of don't exist
            fs::create_dir_all(format!("{}/{}", storage_path, username_hash))
                .map_err(|_| AppError::InternalServerError { msg: "Couldn't create dirs for file".to_string() })?;
            // Save file on disk
            let path = format!("{}/{}/{}", storage_path, username_hash, form.json.filename);
            form.file.file.persist(&path)
                .map_err(|_| AppError::InternalServerError { msg: "Couldn't write file".to_string() })?;

            Ok(HttpResponse::Ok().body(format!("File written to {}", path)))
        },
        None => Err(AppError::Unauthorized)
    }
}

// Downloads the file from authorized user storage
/*#[post("/api/files/download")]
async fn download_file(
    data: web::Data<AppState>,
) -> impl Responder {

}*/
