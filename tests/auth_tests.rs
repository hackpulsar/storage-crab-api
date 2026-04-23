mod common;

use actix_web::test;
use storage_crab::models::jwt::JwtTokenPair;
use serde::Deserialize;
use serde_json::json;

use crate::common::*;

#[derive(Deserialize)]
struct RegisterResponse {
    id: i32,
    email: String,
    username: String
}

#[actix_web::test]
async fn test_auth_flow() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let new_user = create_unique_test_user();

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
    let ctx = setup().await;
    let app = make_app!(ctx);

    let new_user = create_unique_test_user();

    let resp = register(&app, &new_user).await;
    assert_eq!(resp.status(), 200);

    // Try register same user again
    let resp = register(&app, &new_user).await;
    assert!(!resp.status().is_success());
}

#[actix_web::test]
async fn test_login_wrong_password() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let new_user = create_unique_test_user();

    let resp = register(&app, &new_user).await;
    assert_eq!(resp.status(), 200);

    let resp = login(&app, new_user.email, "wrong_password".to_string()).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_login_not_found() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let resp = login(&app, "nonexistant@nowhere.com".to_string(), "phantom_pass".to_string()).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_refresh_invalid_token() {
    let ctx = setup().await;
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
    let ctx = setup().await;
    let app = make_app!(ctx);

    let new_user = create_unique_test_user();

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
    let ctx = setup().await;
    let app = make_app!(ctx);

    let new_user = create_unique_test_user();

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
