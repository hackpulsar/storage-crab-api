mod common;

use actix_web::{dev::{Service, ServiceResponse}, test, web};
use actix_http::Request;
use storage_crab::{models::{
    jwt::JwtTokenPair, 
    user::{DBUser, UserLoginCredentials}}, 
    routes::init_routes
};
use serde::Deserialize;
use serde_json::json;

// Blanket impl, typedef basically
trait TestApp: Service<Request, Response = ServiceResponse, Error = actix_web::Error> {}
impl<T> TestApp for T where T: Service<Request, Response = ServiceResponse, Error = actix_web::Error> {}

#[derive(Deserialize)]
struct RegisterResponse {
    id: i32,
    email: String,
    username: String
}

async fn login(app: &impl TestApp, email: String, password_hash: String) -> ServiceResponse {
    let req = test::TestRequest::post()
        .uri("/api/token/get/")
        .set_json(UserLoginCredentials { email, password_hash })
        .to_request();

    return test::call_service(&app, req).await;
}

async fn register(
    app: &impl TestApp, 
    user: &DBUser
) -> ServiceResponse {
    let req = test::TestRequest::post()
        .uri("/api/users/")
        .set_json(DBUser{ 
            email: user.email.clone(),
            username: user.username.clone(),
            password_hash: user.password_hash.clone()
        })
        .to_request();

    return test::call_service(&app, req).await;
}

#[actix_web::test]
async fn test_auth_flow() {
    let ctx = common::setup().await;
    let app = make_app!(ctx);

    let new_user = DBUser {
        email: "test@test.com".to_string(),
        username: "test".to_string(),
        password_hash: "test".to_string()
    };

    // Step 1. Register
    let resp = register(&app, &new_user).await;
    assert_eq!(resp.status(), 200);

    let body: RegisterResponse = test::read_body_json(resp).await;
    assert!(body.id > -1);
    assert_eq!(body.email, new_user.email);
    assert_eq!(body.username, new_user.username);

    // Step 2. Login
    let resp = login(&app, new_user.email, new_user.password_hash).await;
    assert!(resp.status().is_success());

    let body: JwtTokenPair = test::read_body_json(resp).await;
    assert!(!body.access_token.is_empty());
    assert!(!body.refresh_token.is_empty());

    // Step3. Refresh
    let req = test::TestRequest::post()
        .uri("/api/token/refresh/")
        .set_json(json!({ "refresh_token": body.refresh_token.clone() }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: JwtTokenPair = test::read_body_json(resp).await;
    assert!(!body.access_token.is_empty());
    assert!(!body.refresh_token.is_empty());
}

#[actix_web::test]
async fn test_register_already_exists() {
    let ctx = common::setup().await;
    let app = make_app!(ctx);

    let new_user = DBUser {
        email: "A@a.com".to_string(),
        username: "a".to_string(),
        password_hash: "a".to_string()
    };

    let resp = register(&app, &new_user).await;
    assert_eq!(resp.status(), 200);

    // Try register same user again
    let resp = register(&app, &new_user).await;
    assert!(!resp.status().is_success());
}

#[actix_web::test]
async fn test_login_wrong_password() {
    let ctx = common::setup().await;
    let app = make_app!(ctx);

    let new_user = DBUser {
        email: "test@test.com".to_string(),
        username: "test".to_string(),
        password_hash: "test".to_string()
    };

    let resp = register(&app, &new_user).await;
    assert_eq!(resp.status(), 200);

    let resp = login(&app, new_user.email, "NOTtest".to_string()).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_login_not_found() {
    let ctx = common::setup().await;
    let app = make_app!(ctx);

    let resp = login(&app, "nonexistant@nowhere.com".to_string(), "phantom_pass".to_string()).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_refresh_invalid_token() {
    let ctx = common::setup().await;
    let app = make_app!(ctx);

    let req = test::TestRequest::post()
        .uri("/api/token/refresh/")
        .set_json(json!({ "refresh_token": "banana" }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(!resp.status().is_success());
}

#[actix_web::test]
async fn test_refresh_wrong_type() {
    let ctx = common::setup().await;
    let app = make_app!(ctx);

    let new_user = DBUser {
        email: "test@test.com".to_string(),
        username: "test".to_string(),
        password_hash: "test".to_string()
    };

    let resp = register(&app, &new_user).await;
    assert!(resp.status().is_success());

    let resp = login(&app, new_user.email, new_user.password_hash).await;
    assert!(resp.status().is_success());

    let body: JwtTokenPair = test::read_body_json(resp).await;
    assert!(!body.access_token.is_empty());
    assert!(!body.refresh_token.is_empty());

    let req = test::TestRequest::post()
        .uri("/api/token/refresh/")
        .set_json(json!({ "refresh_token": body.access_token.clone() })) // passing access instead of refresh
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(!resp.status().is_success());
}

#[actix_web::test]
async fn test_refresh_token_blacklisted() {
    let ctx = common::setup().await;
    let app = make_app!(ctx);

    let new_user = DBUser {
        email: "test@test.com".to_string(),
        username: "test".to_string(),
        password_hash: "test".to_string()
    };

    let resp = register(&app, &new_user).await;
    assert!(resp.status().is_success());

    let resp = login(&app, new_user.email, new_user.password_hash).await;
    assert!(resp.status().is_success());

    let body: JwtTokenPair = test::read_body_json(resp).await;
    assert!(!body.access_token.is_empty());
    assert!(!body.refresh_token.is_empty());

    let req = test::TestRequest::post()
        .uri("/api/token/refresh/")
        .set_json(json!({ "refresh_token": body.refresh_token.clone() }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::post()
        .uri("/api/token/refresh/")
        .set_json(json!({ "refresh_token": body.refresh_token.clone() }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}
