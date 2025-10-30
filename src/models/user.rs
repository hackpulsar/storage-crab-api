use serde::{Deserialize, Serialize};

// User record in a database.
// To be used only internally.
#[derive(Serialize, Deserialize)]
pub struct DBUser {
    pub email: String,
    pub username: String,
    pub password_hash: String,
}

// Essential user information.
// To be used externally.
#[derive(Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub email: String,
    pub username: String,
}

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
