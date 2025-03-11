use actix_multipart::form::{MultipartForm};
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use std::path::Path;
use sha2::{Digest, Sha256};
use sqlx::{Row};
use tokio::fs::{self};
use tokio_util::io::ReaderStream;

use crate::AppState;
use crate::models::file::*;
use crate::services::auth::{get_and_validate_jwt};
use crate::utils::errors::AppError;

// Searches for all the files associated with given user
#[get("/api/files/")]
async fn get_files(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let decoded = get_and_validate_jwt(&req, data.secret.clone().as_str())?;

    let user_id: i32 = decoded.claims.sub
        .parse::<i32>()
        .map_err(|_| AppError::InternalServerError { msg: "Sub id parse failed".to_string() })?;

    let res = sqlx::query("select * from files where user_id = $1")
        .bind(user_id)
        .fetch_all(&data.db)
        .await;

    match res {
        Ok(records) => {
            // Convert to a custom File type
            let data: Vec<File> = records
                .into_iter()
                .filter_map(|row| File::from_row(&row).ok())
                .collect();

            // Forming a response with all the files
            Ok(HttpResponse::Ok().json(data))
        }
        Err(_) => Err(AppError::InternalServerError { msg: "Select files query failed".to_string() })
    }
}

// Uploads a file to an authorized user storage
#[post("/api/files/upload/")]
async fn upload_file(
    req: HttpRequest,
    MultipartForm(form): MultipartForm<FileUploadForm>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let decoded = get_and_validate_jwt(&req, data.secret.clone().as_str())?;

    // Retrieve username from db
    let res = sqlx::query("select username from users where id = $1")
        .bind(
            decoded.claims.sub
                .parse::<i32>()
                .map_err(|_| AppError::InternalServerError { msg: "Sub parse failed".to_string() })?
        )
        .fetch_one(&data.db)
        .await;

    let row = res
        .map_err(|_| AppError::InternalServerError { msg: "Username retrieve query failed".to_string() })?;
    let username: String = row.get("username");

    // Hash username with server secret
    let mut hasher = Sha256::new();
    hasher.update(username.as_bytes());
    let username_hash = hex::encode(hasher.finalize());

    // Path to files storage
    let storage_path = std::env::var("FILES_STORAGE_PATH").unwrap();
    let path = format!("{}/{}/{}", storage_path, username_hash, form.json.filename.clone());

    // Check if file already exists
    if Path::new(path.as_str()).exists() {
        return Err(AppError::BadRequest { msg: "File with this name already exists".to_string() });
    }

    // Create dirs for file of don't exist
    fs::create_dir_all(format!("{}/{}", storage_path, username_hash)).await
        .map_err(|_| AppError::InternalServerError { msg: "Couldn't create dirs for file".to_string() })?;
    // Save file on disk
    form.file.file.persist(&path)
        .map_err(|_| AppError::InternalServerError { msg: "Couldn't write file".to_string() })?;

    // Create a new record in files table
    let res = sqlx::query("insert into files (filename, path, size, user_id) values ($1, $2, $3, $4) returning id")
        .bind(form.json.filename.clone())
        .bind(&path)
        .bind(form.file.size as i64)
        .bind(
            decoded.claims.sub
                .parse::<i32>()
                .map_err(|_| AppError::InternalServerError { msg: "File insert query failed".to_string() })?
        )
        .fetch_one(&data.db)
        .await;

    match res {
        Ok(row) => {
            Ok(
                HttpResponse::Ok()
                    .json(FileUploadResponse {
                        file_id: row.get::<i32, _>("id"),
                        path,
                    })
            )
        }
        Err(_) => Err(AppError::InternalServerError { msg: "File upload query failed".to_string() })?
    }
}

// Downloads the file from authorized user storage
#[post("/api/files/download/{file_id}/")]
async fn download_file(
    file_identifier: web::Path<FileIdentifier>,
    req: HttpRequest,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    // Token validation
    let decoded = get_and_validate_jwt(&req, &data.secret)?;

    // Check if the file exists in a db
    let record = File::exists_in_and_belongs_to(
        file_identifier.into_inner().file_id,
        &data.db,
        decoded.claims.sub.parse::<i32>()
            .map_err(|_| AppError::InternalServerError { msg: "Sub id parse failed".to_string() })?
    ).await?;

    let file = fs::File::open(record.get::<String, _>("path")).await
        .map_err(|_| AppError::InternalServerError { msg: "Failed to open file".to_string() })?;
    Ok(
        HttpResponse::Ok()
            .content_type("application/octet-stream")
            .streaming(ReaderStream::new(file))
    )
}

// Deletes the file completely
#[post("/api/files/delete/{file_id}/")]
async fn delete_file(
    file_identifier: web::Path<FileIdentifier>,
    req: HttpRequest,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    // Token validation
    let decoded = get_and_validate_jwt(&req, &data.secret)?;
    let file_id = file_identifier.into_inner().file_id;

    // Check if the file exists in a db
    let record = File::exists_in_and_belongs_to(
        file_id,
        &data.db,
        decoded.claims.sub.parse::<i32>()
            .map_err(|_| AppError::InternalServerError { msg: "Sub id parse failed".to_string() })?
    ).await?;

    // Delete the file from storage
    let path: String = record.get("path");
    tokio::fs::remove_file(&path).await
        .map_err(|_| AppError::InternalServerError { msg: "Failed to delete file".to_string() })?;

    // Remove the record in db
    let _ = sqlx::query("delete from files where id = $1")
        .bind(file_id)
        .execute(&data.db)
        .await
        .map_err(|_| AppError::InternalServerError { msg: "Failed to delete file".to_string() })?;

    Ok(HttpResponse::Ok()
        .body(format!("File {} deleted successfully.", path))
    )
}
