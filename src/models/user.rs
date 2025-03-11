use serde::{Deserialize, Serialize};

// Struct for user registration
#[derive(Serialize, Deserialize)]
pub struct RegisterUser {
    pub email: String,
    pub username: String,
    pub password_hash: String,
}

// Response to a create user request
#[derive(Serialize)]
pub struct RegisterUserResponse {
    pub id: i32,
    pub user: RegisterUser,
}

// User credentials on login
#[derive(Deserialize)]
pub struct UserLoginCredentials {
    pub email: String,
    pub password_hash: String,
}

impl UserLoginCredentials {
    // Compares given password hash to user password hash
    pub fn verify_password(&self, password_hash: &str) -> bool {
        self.password_hash == password_hash
    }
}
