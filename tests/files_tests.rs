mod common;

use actix_web::{dev::ServiceResponse, test};
use actix_multipart_test::MultiPartFormDataBuilder;
use storage_crab::models::file::{DBFile, FileShareResponse, FileUploadResponse};
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

    let credentials = sign_in_new_user(&app).await;
        
    let resp = upload_test_file(&app, credentials.tokens.access_token, None).await;
    assert!(resp.status().is_success());

    let details: FileUploadResponse = test::read_body_json(resp).await;
    assert!(std::path::Path::new(&details.path).exists());
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

    let credentials = sign_in_new_user(&app).await;
    let custom_filename = Some("test_upload_file_exists".to_string());
        
    let resp = upload_test_file(
        &app, credentials.tokens.access_token.clone(), custom_filename.clone()
    ).await;
    assert!(resp.status().is_success());

    // Uploading same file again
    let resp = upload_test_file(
        &app, credentials.tokens.access_token, custom_filename
    ).await;
    assert!(!resp.status().is_success());
}

#[actix_web::test]
async fn test_get_files() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let credentials = sign_in_new_user(&app).await;
    let filenames = vec!["1", "2", "3"];

    // Uploading files
    for name in &filenames {
        let resp = upload_test_file(
            &app, credentials.tokens.access_token.clone(), Some(name.to_string())
        ).await;
        assert!(resp.status().is_success());
    }

    // Retrieving files
    let req = test::TestRequest::get()
        .uri("/api/files/")
        .insert_header(("Authorization", format!("Bearer {}", credentials.tokens.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let files: Vec<DBFile> = test::read_body_json(resp).await;

    for (i, file) in files.iter().enumerate() {
        assert_eq!(file.filename, filenames[i]);
    }

}

#[actix_web::test]
async fn test_get_files_unauthorized() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let req = test::TestRequest::get()
        .uri("/api/files/")
        .insert_header(("Authorization", "Bearer banana"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_download_file() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let credentials = sign_in_new_user(&app).await;

    // Upload a file
    let resp = upload_test_file(
        &app, credentials.tokens.access_token.clone(), Some("test_download_file.jpg".to_string())
    ).await;
    assert!(resp.status().is_success());

    let file_data: FileUploadResponse = test::read_body_json(resp).await;

    let req = test::TestRequest::get()
        .uri(format!("/api/files/download/{}/", file_data.file_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", credentials.tokens.access_token.clone())))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Check header content type
    let content_type = resp.headers().get("Content-Type").unwrap();
    assert_eq!(content_type, "application/octet-stream");

    // Check filename
    let disposition = resp.headers().get("Content-Disposition").unwrap();
    assert!(disposition.to_str().unwrap().contains("test_download_file.jpg"));

    // Check body matches what was uploaded
    let original_bytes = std::fs::read("tests/fixtures/test.jpg").unwrap();
    let body_bytes = test::read_body(resp).await;
    assert_eq!(body_bytes, original_bytes);
}

#[actix_web::test]
async fn test_download_not_found() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let credentials = sign_in_new_user(&app).await;

    let req = test::TestRequest::get()
        .uri(format!("/api/files/download/{}/", 67).as_str())
        .insert_header(("Authorization", format!("Bearer {}", credentials.tokens.access_token.clone())))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_download_unauthorized() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let req = test::TestRequest::get()
        .uri(format!("/api/files/download/{}/", 67).as_str())
        .insert_header(("Authorization", "Bearer banana"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_delete_file() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let credentials = sign_in_new_user(&app).await;

    // Upload a file
    let resp = upload_test_file(
        &app, credentials.tokens.access_token.clone(), None
    ).await;
    assert!(resp.status().is_success());

    let file_data: FileUploadResponse = test::read_body_json(resp).await;

    let req = test::TestRequest::post()
        .uri(format!("/api/files/delete/{}/", file_data.file_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", credentials.tokens.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify file was really deleted
    let req = test::TestRequest::get()
        .uri(format!("/api/files/download/{}/", file_data.file_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", credentials.tokens.access_token.clone())))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    assert!(!std::path::Path::new(&file_data.path).exists());
}

#[actix_web::test]
async fn test_delete_file_not_found() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let credentials = sign_in_new_user(&app).await;

    let req = test::TestRequest::post()
        .uri(format!("/api/files/delete/{}/", 67).as_str())
        .insert_header(("Authorization", format!("Bearer {}", credentials.tokens.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_delete_unauthorized() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let req = test::TestRequest::post()
        .uri(format!("/api/files/delete/{}/", 67).as_str())
        .insert_header(("Authorization", "Bearer banana"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_share_file() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let credentials = sign_in_new_user(&app).await;

    let resp = upload_test_file(
        &app, credentials.tokens.access_token.clone(), None
    ).await;
    assert!(resp.status().is_success());

    let file_data: FileUploadResponse = test::read_body_json(resp).await;

    let req = test::TestRequest::post()
        .uri(format!("/api/files/share/{}/", file_data.file_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", credentials.tokens.access_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let share_details: FileShareResponse = test::read_body_json(resp).await;
    assert!(!share_details.code.is_empty());
    assert_eq!(share_details.code.len(), 8);
}

#[actix_web::test]
async fn test_share_file_not_found() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let credentials = sign_in_new_user(&app).await;

    let req = test::TestRequest::post()
        .uri(format!("/api/files/share/{}/", 67).as_str())
        .insert_header(("Authorization", format!("Bearer {}", credentials.tokens.access_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_share_file_unauthorized() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let req = test::TestRequest::post()
        .uri(format!("/api/files/share/{}/", 67).as_str())
        .insert_header(("Authorization", "Bearer banana"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_download_shared() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    // User A
    let user_a = sign_in_new_user(&app).await;

    // Upload a file
    let resp = upload_test_file(
        &app, user_a.tokens.access_token.clone(), Some("test_download_file.jpg".to_string())
    ).await;
    assert!(resp.status().is_success());

    let file_data: FileUploadResponse = test::read_body_json(resp).await;

    // Share a file
    let req = test::TestRequest::post()
        .uri(format!("/api/files/share/{}/", file_data.file_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", user_a.tokens.access_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let share_details: FileShareResponse = test::read_body_json(resp).await;

    // User B
    let user_b = sign_in_new_user(&app).await;

    let req = test::TestRequest::get()
        .uri(format!("/api/files/download/shared/{}/", share_details.code).as_str())
        .insert_header(("Authorization", format!("Bearer {}", user_b.tokens.access_token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Check header content type
    let content_type = resp.headers().get("Content-Type").unwrap();
    assert_eq!(content_type, "application/octet-stream");

    // Check filename
    let disposition = resp.headers().get("Content-Disposition").unwrap();
    println!("{}", disposition.to_str().unwrap());
    assert!(disposition.to_str().unwrap().contains("test_download_file.jpg"));

    // Check body matches what was shared
    let original_bytes = std::fs::read(file_data.path).unwrap();
    let body_bytes = test::read_body(resp).await;
    assert_eq!(body_bytes, original_bytes);
}

#[actix_web::test]
async fn test_download_shared_invalid_code() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let credentials = sign_in_new_user(&app).await;

    let req = test::TestRequest::get()
        .uri(format!("/api/files/download/shared/{}/", "banana").as_str())
        .insert_header(("Authorization", format!("Bearer {}", credentials.tokens.access_token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(!resp.status().is_success());
}

#[actix_web::test]
async fn test_download_shared_expired_code() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    // User A
    let user_a = sign_in_new_user(&app).await;

    // Upload a file
    let resp = upload_test_file(
        &app, user_a.tokens.access_token.clone(), None
    ).await;
    assert!(resp.status().is_success());

    let file_data: FileUploadResponse = test::read_body_json(resp).await;

    // Share a file
    let req = test::TestRequest::post()
        .uri(format!("/api/files/share/{}/", file_data.file_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", user_a.tokens.access_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let share_details: FileShareResponse = test::read_body_json(resp).await;

    // Share the same file again, to expire previous code
    let req = test::TestRequest::post()
        .uri(format!("/api/files/share/{}/", file_data.file_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", user_a.tokens.access_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // User B
    let user_b = sign_in_new_user(&app).await;

    let req = test::TestRequest::get()
        .uri(format!("/api/files/download/shared/{}/", share_details.code).as_str())
        .insert_header(("Authorization", format!("Bearer {}", user_b.tokens.access_token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(!resp.status().is_success());
}