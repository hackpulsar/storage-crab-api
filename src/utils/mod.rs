use std::process::Command;

pub mod errors;

// Generates shared secret using OpenSSL
pub fn generate_shared_secret() -> String {
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