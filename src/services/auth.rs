use actix_web::HttpRequest;
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};

use crate::models::jwt::Claims;
use crate::utils::errors::AppError;

// Reads token from request header
pub fn get_jwt_from(req: &HttpRequest) -> Result<&str, AppError> {
    let header = req.headers().get("Authorization");
    match header {
        Some(header) => {
            let header = header.to_str().map_err(|_| AppError::BadRequest {
                msg: "Failed to parse header".to_string(),
            })?;
            if let Some(token) = header.strip_prefix("Bearer ") {
                Ok(token)
            } else {
                Err(AppError::BadRequest {
                    msg: "Missing Bearer token".to_string(),
                })
            }
        }
        None => Err(AppError::BadRequest {
            msg: "Missing Authorization field in header".to_string(),
        }),
    }
}

// Validates the JWT
pub fn validate_jwt(token: &str, secret: &str) -> Option<TokenData<Claims>> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .ok()
}

pub fn get_and_validate_jwt(
    req: &HttpRequest,
    secret: &str,
) -> Result<TokenData<Claims>, AppError> {
    // Get the token
    let token = get_jwt_from(req).map_err(|_| AppError::BadRequest {
        msg: "Failed to parse token".to_string(),
    })?;
    // Validate it
    Ok(validate_jwt(token, secret).ok_or(AppError::Unauthorized)?)
}
