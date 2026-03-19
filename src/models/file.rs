use actix_multipart::form::{json::Json, tempfile::TempFile, MultipartForm};
use chrono::{NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::{Error, Pool, Postgres, Row};
use sqlx::postgres::PgRow;
use log::{warn, debug};

use crate::utils::errors::AppError;

#[derive(Debug, Deserialize)]
pub struct FileMetadata {
    pub filename: String,
}

#[derive(Debug, MultipartForm)]
pub struct FileUploadForm {
    #[multipart(limit = "100MB")]
    pub file: TempFile,
    pub json: Json<FileMetadata>,
}

// A file in database
#[derive(Serialize)]
pub struct DBFile {
    pub id: i32,
    pub filename: String,
    pub path: String,
    pub size: i64,
    pub uploaded_at: NaiveDateTime,
    pub user_id: i32
}

#[derive(Serialize)]
pub struct FileUploadResponse {
    pub file_id: i32,
    pub path: String
}

#[derive(Serialize)]
pub struct FileShareResponse {
    pub code: String
}

impl DBFile {
    // Extracts file from a record in db
    pub fn from_row(row: &PgRow) -> Result<Self, Error> {
        Ok(DBFile {
            id: row.get("id"),
            filename: row.get("filename"),
            path: row.get("path"),
            size: row.get("size"),
            uploaded_at: row.get("uploaded_at"),
            user_id: row.get("user_id")
        })
    }

    pub async fn exists_and_belongs_to(
        id: i32,
        db: &Pool<Postgres>,
        user_id: i32
    ) -> Result<PgRow, AppError> {
        let res = sqlx::query("select path, filename, size from files where id = $1 and user_id = $2")
            .bind(id)
            .bind(user_id)
            .fetch_optional(db)
            .await
            .map_err(|e| {
                warn!("Failed to fetch file with error {:?}", e);
                AppError::InternalServerError { msg: format!("Failed to fetch file with error {:?}", e) }
            })?;

        match res {
            Some(record) => {
                debug!("Found file with ID [{}]", id);
                Ok(record)
            }
            None => {
                debug!("File with ID [{}] foesn't exist", id);
                return Err(AppError::InternalServerError { msg: "File doesn't exist".to_string() })
            }
        }
    }

    pub async fn exists(
        id: i32,
        db: &Pool<Postgres>
    ) -> Result<PgRow, AppError> {
        let res = sqlx::query("select path, filename, size from files where id = $1")
            .bind(id)
            .fetch_optional(db)
            .await
            .map_err(|e| {
                warn!("Failed to fetch file with error {:?}", e);
                AppError::InternalServerError { msg: format!("Failed to fetch file with error {:?}", e) }
            })?;

        match res {
            Some(record) => {
                debug!("Found file with ID [{}]", id);
                Ok(record)
            }
            None => {
                debug!("File with ID [{}] foesn't exist", id);
                return Err(AppError::InternalServerError { msg: "File doesn't exist".to_string() })
            }
        }
    }

}
