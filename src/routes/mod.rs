use actix_web::web;

pub mod auth;
pub mod files;

// Initializes routes for server config
pub fn init_routes(config: &mut web::ServiceConfig) {
    // Auth endpoints
    config.service(auth::login);
    config.service(auth::refresh_token);
    config.service(auth::greet);

    // Files endpoints
    config.service(files::upload_file);
}