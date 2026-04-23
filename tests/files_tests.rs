mod common;

use actix_web::{dev::ServiceResponse, test};
use actix_multipart_test::MultiPartFormDataBuilder;
use storage_crab::models::jwt::JwtTokenPair;
use uuid::Uuid;

use crate::common::*;

async fn upload_test_file(app: &impl TestApp, access_token: String, custom_name: Option<String>) -> ServiceResponse {
    let mut builder = MultiPartFormDataBuilder::new();
    let filename = custom_name.unwrap_or(format!("test_{}", Uuid::new_v4()));
    builder.with_file("tests/fixtures/test.jpg", "file", "image/jpg", "test.jpg");
    builder.with_custom_text("json", format!("{{\"filename\": \"{}\"}}", filename), "application/json");
    let (header, body) = builder.build();
        
    let req = test::TestRequest::post()
        .uri("/api/files/upload/")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .insert_header(header)
        .set_payload(body)
        .to_request();

    return test::call_service(&app, req).await;
}

#[actix_web::test]
async fn test_upload_file() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let new_user = create_unique_test_user();

    // Register & login
    let resp = register(&app, &new_user).await;
    assert!(resp.status().is_success());
    let resp = login(&app, new_user.email, new_user.password_hash).await;
    assert!(resp.status().is_success());

    let tokens: JwtTokenPair = test::read_body_json(resp).await;
    assert!(!tokens.access_token.is_empty());
    assert!(!tokens.refresh_token.is_empty());
        
    let resp = upload_test_file(&app, tokens.access_token, None).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_upload_unauthorized() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let resp = upload_test_file(&app, "banana".to_string(), None).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_upload_file_exists() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let new_user = create_unique_test_user();

    // Register & login
    let resp = register(&app, &new_user).await;
    assert!(resp.status().is_success());
    let resp = login(&app, new_user.email, new_user.password_hash).await;
    assert!(resp.status().is_success());

    let tokens: JwtTokenPair = test::read_body_json(resp).await;
    assert!(!tokens.access_token.is_empty());
    assert!(!tokens.refresh_token.is_empty());

    let custom_filename = Some("test_upload_file_exists".to_string());
        
    let resp = upload_test_file(
        &app, tokens.access_token.clone(), custom_filename.clone()
    ).await;
    assert!(resp.status().is_success());

    // Uploading same file again
    let resp = upload_test_file(
        &app, tokens.access_token, custom_filename
    ).await;
    assert!(!resp.status().is_success());
}

#[actix_web::test]
async fn test_get_files() {}

#[actix_web::test]
async fn test_download_file() {}

#[actix_web::test]
async fn test_download_not_found() {}

#[actix_web::test]
async fn tests_download_shared() {}

#[actix_web::test]
async fn test_download_shared_invalid_code() {}

#[actix_web::test]
async fn test_delete_file() {}

#[actix_web::test]
async fn test_delete_file_not_found() {}

#[actix_web::test]
async fn test_share_file() {}

#[actix_web::test]
async fn test_share_file_not_found() {}

#[actix_web::test]
async fn test_download_unauthorized() {}

#[actix_web::test]
async fn test_delete_unauthorized() {}

#[actix_web::test]
async fn test_get_files_unauthorized() {}

#[actix_web::test]
async fn test_share_file_unauthorized() {}

#[actix_web::test]
async fn test_download_shared_expired_code() {}