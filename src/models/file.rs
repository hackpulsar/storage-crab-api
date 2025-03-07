use actix_multipart::form::{json::Json, tempfile::TempFile, MultipartForm};
use serde::{Deserialize};

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