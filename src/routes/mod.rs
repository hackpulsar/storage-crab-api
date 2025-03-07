use actix_web::web;

pub mod auth;

// Initializes routes for server config
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(auth::login);
    config.service(auth::refresh_token);
    config.service(auth::greet);
}