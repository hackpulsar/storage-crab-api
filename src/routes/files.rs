use actix_multipart::form::{MultipartForm};
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use log::{warn, debug};
use redis::AsyncCommands;
use std::path::Path;
use sha2::{Digest, Sha256};
use sqlx::{Row};
use tokio::fs::{self};
use tokio_util::io::ReaderStream;

use rand::{distributions::Alphanumeric, Rng};

use crate::AppState;
use crate::models::file::*;
use crate::services::auth::{get_and_validate_jwt};
use crate::utils::errors::AppError;

// Searches for all the files associated with given user
#[get("/api/files/")]
async fn get_files(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let token = get_and_validate_jwt(&req, &data.secret)?;
    let user_id: i32 = token.claims.sub
        .parse::<i32>()
        .map_err(|e| {
            warn!("Sub id parse failed with error: {:?}", e);
            AppError::InternalServerError { msg: "Sub id parse failed".to_string() }
        })?;

    let records = sqlx::query("select * from files where user_id = $1")
        .bind(user_id)
        .fetch_all(&data.db_pool)
        .await
        .map_err(|_| {
            warn!("Select files query failed for user_id: {}", user_id);
            AppError::InternalServerError { msg: "Select files query failed".to_string() }
        })?;

    // Convert to a custom DBFile type
    let data: Vec<DBFile> = records
        .into_iter()
        .filter_map(|row| DBFile::from_row(&row).ok())
        .collect();

    debug!("Files fetched {} files for user: {}", data.len(), user_id);

    Ok(HttpResponse::Ok().json(data))
}

// Uploads a file to an authorized user storage
#[post("/api/files/upload/")]
async fn upload_file(
    req: HttpRequest,
    MultipartForm(form): MultipartForm<FileUploadForm>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let token = get_and_validate_jwt(&req, data.secret.clone().as_str())?;
    let user_id: i32 = token.claims.sub
        .parse::<i32>()
        .map_err(|e| {
            warn!("Sub id parse failed with error: {:?}", e);
            AppError::InternalServerError { msg: "Sub id parse failed".to_string() }
        })?;

    // Retrieve username
    let username: String = sqlx::query("select username from users where id = $1")
        .bind(user_id)
        .fetch_one(&data.db_pool)
        .await
        .map_err(|e| {
            warn!("Username retrieve query failed for user {} with error: {:?}", user_id, e);
            AppError::InternalServerError { msg: "Username retrieve query failed".to_string() }
        })?
        .get("username");

    // Hash username with server secret
    let mut hasher = Sha256::new();
    hasher.update(username.as_bytes());
    let username_hash = hex::encode(hasher.finalize());

    // Path to files storage
    let storage_path = std::env::var("FILES_STORAGE_PATH").unwrap();
    let path = format!("{}/{}/{}", storage_path, username_hash, form.json.filename.clone());

    // Check if file already exists
    if Path::new(path.as_str()).exists() {
        debug!("File [{}] already exists", path);
        return Err(AppError::BadRequest { msg: "File with this name already exists".to_string() });
    }

    let dir_path = format!("{}/{}", storage_path, username_hash);

    // Create dirs for file of don't exist
    fs::create_dir_all(&dir_path).await
        .map_err(|_| {
            warn!("Couldn't create dir [{}]", &dir_path);
            AppError::InternalServerError { msg: "Couldn't create dirs for file".to_string() }
        })?;
    // Save file on disk
    form.file.file.persist(&path)
        .map_err(|e| {
            warn!("Couldn't write file to [{}] with error: {:?}", &path, e);
            AppError::InternalServerError { msg: "Couldn't write file".to_string() }
        })?;

    // Create a new record in files table
    let record = sqlx::query("insert into files (filename, path, size, user_id) values ($1, $2, $3, $4) returning id")
        .bind(form.json.filename.clone())
        .bind(&path)
        .bind(form.file.size as i64)
        .bind(user_id)
        .fetch_one(&data.db_pool)
        .await
        .map_err(|e| {
            warn!("File upload query failed with error: {:?}", e);
            AppError::InternalServerError { msg: "File upload query failed".to_string() }
        })?;

    let file_id = record.get::<i32, _>("id");
    debug!("File with ID [{}] uploaded to [{}]", file_id, path);

    Ok(HttpResponse::Ok().json(FileUploadResponse { file_id: file_id, path }))
}

// Downloads the file from authorized user storage
#[get("/api/files/download/{file_id}/")]
async fn download_file(file_id: web::Path<i32>, req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    // Token validation
    let token = get_and_validate_jwt(&req, &data.secret)?;
    let user_id: i32 = token.claims.sub
        .parse::<i32>()
        .map_err(|e| {
            warn!("Sub id parse failed with error: {:?}", e);
            AppError::InternalServerError { msg: "Sub id parse failed".to_string() }
        })?;

    // Check if the file exists in a db
    let record = DBFile::exists_and_belongs_to(
        file_id.into_inner(),
        &data.db_pool,
        user_id
    ).await?;

    let path: String = record.get("path");
    let file = fs::File::open(&path)
        .await
        .map_err(|e| {
            warn!("Failed to open file [{}] with error: {:?}", path, e);
            AppError::InternalServerError { msg: "Failed to open file".to_string() }
        })?;

    debug!("File downloaded from path [{}]", path);

    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", record.get::<String, _>("filename"))
        ))
        .insert_header((
            "Content-Length",
            record.get::<i64, _>("size")
        ))
        .streaming(ReaderStream::new(file))
    )
}

#[get("/api/files/download/shared/{share_code}/")]
async fn download_shared_file(share_code: web::Path<String>, req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    get_and_validate_jwt(&req, &data.secret)?;

    let mut conn = data.redis_pool
        .get().await
        .map_err(|_| {
            warn!("Connection to Redis lost");
            AppError::InternalServerError { msg: "Connection to Redis lost".to_string() }
        })?;

    // Validate the share code
    let file_id = match conn.get::<_, Option<i32>>(share_code.clone()).await {
        Ok(Some(id)) => {
            debug!("File ID valid: {}", id);
            id
        }
        Ok(None) => {
            debug!("Invalid share code [{}]", share_code);
            return Err(AppError::BadRequest { msg: "Invalid share code".to_string() })
        }
        Err(_) => {
            debug!("Failed to fetch share code from Redis");
            return Err(AppError::InternalServerError { msg: "Failed to fetch share code from Redis".to_string() })
        }
    };

    // Download the file
    let record = DBFile::exists(file_id, &data.db_pool).await?;

    let path: String = record.get("path");
    let file = fs::File::open(&path)
        .await
        .map_err(|e| {
            warn!("Failed to open file [{}] with error: {:?}", path, e);
            AppError::InternalServerError { msg: "Failed to open file".to_string() }
        })?;

    debug!("Shared file downloaded from [{}] with code [{}]", path, share_code);

    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", record.get::<String, _>("filename"))
        ))
        .insert_header((
            "Content-Length",
            record.get::<i64, _>("size")
        ))
        .streaming(ReaderStream::new(file))
    )
}

// Deletes the file completely
#[post("/api/files/delete/{file_id}/")]
async fn delete_file(file_identifier: web::Path<i32>, req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let token = get_and_validate_jwt(&req, &data.secret)?;
    let user_id: i32 = token.claims.sub
        .parse::<i32>()
        .map_err(|e| {
            warn!("Sub id parse failed with error: {:?}", e);
            AppError::InternalServerError { msg: "Sub id parse failed".to_string() }
        })?;
    let file_id = file_identifier.into_inner();

    // Check if the file exists in a db
    let record = DBFile::exists_and_belongs_to(
        file_id,
        &data.db_pool,
        user_id
    ).await?;

    // Delete the file from storage
    let path: String = record.get("path");
    tokio::fs::remove_file(&path)
        .await
        .map_err(|e| {
            warn!("Failed to delete file with error: {:?}", e);
            AppError::InternalServerError { msg: "Failed to delete file".to_string() }
        })?;

    // Remove the record in db
    let _ = sqlx::query("delete from files where id = $1")
        .bind(file_id)
        .execute(&data.db_pool)
        .await
        .map_err(|_| AppError::InternalServerError { msg: "File delete query failed".to_string() })?;

    debug!("File deleted from [{}]", path);

    // An empty OK response
    Ok(HttpResponse::NoContent().finish())
}

#[post("/api/files/share/{file_id}/")]
async fn share_file(file_id: web::Path<i32>, req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let token = get_and_validate_jwt(&req, &data.secret)?;
    let user_id: i32 = token.claims.sub
        .parse::<i32>()
        .map_err(|e| {
            warn!("Sub id parse failed with error: {:?}", e);
            AppError::InternalServerError { msg: "Sub id parse failed".to_string() }
        })?;
    let file_id = file_id.into_inner();

    // Verify if file exists
    DBFile::exists_and_belongs_to(
        file_id,
        &data.db_pool,
        user_id
    ).await?;

    let mut share_code: String;

    // Generate a unique token of length 8
    // Map token to a file in file_shares table
    loop {
        share_code = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        let mut conn = data.redis_pool
            .get().await
            .map_err(|_| {
                warn!("Connection to Redis lost");
                AppError::InternalServerError { msg: "Connection to Redis lost".to_string() }
            })?;

        let key: &str = &share_code;
        let value: Option<String> = conn.get(key).await.ok();

        match value {
            Some(_) => { /* Key already exists, continue */ },
            None => {
                // Code is unique, add it to redis
                conn.set_ex::<_, _, ()>(
                    key,
                    file_id,
                    5 * 60 // 5 minutes
                ).await
                .map_err(|_| {
                    warn!("Failed to save share code [{}]", share_code);
                    AppError::InternalServerError { msg: "Failed to save share code".to_string() }
                })?;
                
                break;
            }
        }
    }

    // Pass the generated code to user
    Ok(HttpResponse::Ok().json(FileShareResponse{ code: share_code }))
}
