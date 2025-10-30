use actix_web::HttpRequest;
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};

use crate::models::jwt::Claims;
use crate::utils::errors::AppError;

// Parses token from request
pub fn get_jwt_from(request: &HttpRequest) -> Result<&str, AppError> {
    let header_value = request
        .headers()
        .get("Authorization")
        .ok_or_else(|| AppError::BadRequest {
            msg: "Missing Authorization field in header".to_string(),
        })?
        .to_str()
        .map_err(|_| AppError::BadRequest {
                msg: "Failed to parse header".to_string(),
        })?;

    header_value
        .strip_prefix("Bearer ")
        .ok_or(AppError::BadRequest { 
                msg: "Missing Bearer token".to_string() 
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
    validate_jwt(token, secret).ok_or(AppError::Unauthorized)
}
