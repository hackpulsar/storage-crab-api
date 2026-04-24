mod common;

use actix_web::test;
use storage_crab::models::user::UserInfo;
use crate::common::*;

#[actix_web::test]
async fn test_user_info() {
    let ctx = setup().await;
    let app = make_app!(ctx);

    let credentials = sign_in_new_user(&app).await;

    let req = test::TestRequest::get()
        .uri("/api/users/me/")
        .insert_header(("Authorization", format!("Bearer {}", credentials.tokens.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let info: UserInfo = test::read_body_json(resp).await;

    assert_eq!(credentials.user.email, info.email);
    assert_eq!(credentials.user.username, info.username)
}