use actix_web::HttpRequest;
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use log::{debug};

use crate::models::jwt::Claims;
use crate::utils::errors::AppError;

// Parses token from request
pub fn get_jwt_from(request: &HttpRequest) -> Result<&str, AppError> {
    let header_value = request
        .headers()
        .get("Authorization")
        .ok_or_else(|| {
            debug!("Missing Authorization field in header");
            AppError::BadRequest { msg: "Missing Authorization field in header".to_string() }
        })?
        .to_str()
        .map_err(|_| {
            debug!("Failed to parse header");
            AppError::BadRequest { msg: "Failed to parse header".to_string() }
        })?;

    header_value
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            debug!("Missing Bearer token in header");
            AppError::BadRequest { msg: "Missing Bearer token".to_string() }
        })
}

// Validates the JWT
pub fn validate_jwt(token: &str, secret: &str) -> Option<TokenData<Claims>> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ).ok()
}

pub fn get_and_validate_jwt(req: &HttpRequest, secret: &str) -> Result<TokenData<Claims>, AppError> {
    let token = get_jwt_from(req)?;
    validate_jwt(token, secret).ok_or_else(|| {
        debug!("Unathorized access with token [{}]", token);
        AppError::Unauthorized
    })
}
