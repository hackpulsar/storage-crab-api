use serde::{Deserialize, Serialize};

// User record in a database.
// To be used only internally.
#[derive(Serialize, Deserialize, Debug)]
pub struct DBUser {
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
}

// Essential user information.
// To be used externally.
#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i32,
    pub email: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRegisterCredentials {
    pub email: String,
    pub username: String,
    pub password: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserLoginCredentials {
    pub email: String,
    pub password: String,
}
