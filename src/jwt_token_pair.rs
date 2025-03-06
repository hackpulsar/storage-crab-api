use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::process::Command;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,            // subject (whom token refers to)
    pub exp: usize,             // token expiration date
    pub token_type: TokenType,  // token type (access or refresh)
    pub jti: String,            // token id
}

// Represents a token pair
#[derive(Serialize)]
pub struct JwtTokenPair {
    access_token: String,
    refresh_token: String,
}

// Token types
#[derive(Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

impl JwtTokenPair {
    // Generates a new token pair with the following lifetime
    // Access: 10 minutes
    // Refresh: 30 minutes
    pub fn generate_for(user_email: String, secret: String) -> Self {
        let access_exp = chrono::Utc::now() + chrono::Duration::minutes(10);
        let refresh_exp = chrono::Utc::now() + chrono::Duration::minutes(30);

        let access_claims = Claims {
            sub: user_email.clone(),
            exp: access_exp.timestamp() as usize,
            token_type: TokenType::Access,
            jti: Uuid::new_v4().to_string(),
        };
        let refresh_claims = Claims {
            sub: user_email.clone(),
            exp: refresh_exp.timestamp() as usize,
            token_type: TokenType::Refresh,
            jti: Uuid::new_v4().to_string(),
        };

        // Return a freshly generated token pair
        Self {
            access_token: encode(
                &Header::default(),
                &access_claims,
                &EncodingKey::from_secret(secret.as_bytes())
            ).unwrap(),
            refresh_token: encode(
                &Header::default(),
                &refresh_claims,
                &EncodingKey::from_secret(secret.as_bytes())
            ).unwrap()
        }
    }
}

// Generates shared secret using openssl
pub fn generate_shared_secret() -> String{
    String::from_utf8(Command::new("openssl")
        .arg("rand")
        .arg("-base64")
        .arg("32")
        .output()
        .expect("Failed to generate shared secret. Make sure openssl is installed.")
        .stdout
    )
    .unwrap()
    .trim()
    .to_string()
}
