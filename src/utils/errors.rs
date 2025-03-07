use actix_web::{HttpResponse, ResponseError};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use derive_more::Display;

// Errors tha occur during app runtime
#[derive(Debug, Display)]
pub enum AppError {
    #[display("Unauthorized access")]
    Unauthorized,

    #[display("Bad request: {}", msg)]
    BadRequest { msg: String },

    #[display("Internal server error: {}", msg)]
    InternalServerError { msg: String },
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            AppError::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }
}
