use serde::{Deserialize, Serialize};

// User record in a database
#[derive(Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub username: String,
    pub password_hash: String,
}

// Response to a create user request
// TODO: do we need the entire user with even password hash?
#[derive(Serialize)]
pub struct RegisterUserResponse {
    pub id: i32,
    pub user: User,
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

// Response to the "me" endpoint
#[derive(Serialize)]
pub struct UserInfoReponse {
    pub email: String,
    pub username: String,
}
