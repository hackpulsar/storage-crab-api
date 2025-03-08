use actix_multipart::form::{json::Json, tempfile::TempFile, MultipartForm};
use chrono::{NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::{Error, Row};
use sqlx::postgres::PgRow;

// Form of metadata for file upload
#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub filename: String,
}

// Upload form of a file
#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "100MB")]
    pub file: TempFile,
    pub json: Json<Metadata>,
}

// A file in database
#[derive(Serialize)]
pub struct File {
    pub id: i32,
    pub filename: String,
    pub path: String,
    pub size: i64,
    pub uploaded_at: NaiveDateTime,
    pub user_id: i32
}

impl File {
    pub fn from_row(row: &PgRow) -> Result<Self, Error> {
        Ok(File{
            id: row.get("id"),
            filename: row.get("filename"),
            path: row.get("path"),
            size: row.get("size"),
            uploaded_at: row.get("uploaded_at"),
            user_id: row.get("user_id")
        })
    }
}
