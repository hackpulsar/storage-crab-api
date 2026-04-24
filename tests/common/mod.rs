use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::{postgres::Postgres, redis::Redis};
use storage_crab::{AppState, create_db_pool, create_redis_pool, models::jwt::JwtTokenPair};
use tempfile::TempDir;

use actix_web::{dev::{Service, ServiceResponse}, test};
use actix_http::Request;
use storage_crab::{models::user::{DBUser, UserLoginCredentials}};
use uuid::Uuid;

// Blanket impl, typedef basically
pub trait TestApp: Service<Request, Response = ServiceResponse, Error = actix_web::Error> {}
impl<T> TestApp for T where T: Service<Request, Response = ServiceResponse, Error = actix_web::Error> {}

pub struct TestContext {
    pub state: AppState,

    // Temporary files storage path while writing
    pub temp_dir: TempDir,

    // Containers wrappers
    _pg: ContainerAsync<Postgres>,
    _redis: ContainerAsync<Redis>
}

pub async fn setup() -> TestContext {
    let pg = Postgres::default().start().await.unwrap();
    let redis = Redis::default().start().await.unwrap();

    let pg_url = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        pg.get_host_port_ipv4(5432).await.unwrap()
    );
    let redis_url = format!(
        "redis://127.0.0.1:{}",
        redis.get_host_port_ipv4(6379).await.unwrap()
    );

    let db_pool = create_db_pool(pg_url).await.unwrap();
    let redis_pool = create_redis_pool(redis_url).unwrap();

    sqlx::migrate!().run(&db_pool).await.unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    let storage_dir = tempfile::tempdir().unwrap();
    std::env::set_var("FILES_STORAGE_PATH", storage_dir.path());

    TestContext {
        state: AppState {
            secret: "test-secret".into(),
            db_pool,
            redis_pool,
            storage_dir: storage_dir.path().to_str().unwrap().to_string()
        },
        temp_dir,
        _pg: pg,
        _redis: redis,
    }
}

pub async fn login(app: &impl TestApp, email: &str, password_hash: &str) -> ServiceResponse {
    let req = test::TestRequest::post()
        .uri("/api/token/get/")
        .set_json(UserLoginCredentials { 
            email: email.to_string(),
            password_hash: password_hash.to_string()
        })
        .to_request();

    return test::call_service(&app, req).await;
}

pub async fn register(
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

pub fn create_unique_test_user() -> DBUser {
    return DBUser {
        email: format!("{}@test.com", Uuid::new_v4()).to_string(),
        username: "test".to_string(),
        password_hash: "test".to_string()
    };
}

#[allow(dead_code)] // Ignore for test helper
pub struct Credentials {
    pub user: DBUser,
    pub tokens: JwtTokenPair
}

pub async fn sign_in_new_user(app: &impl TestApp) -> Credentials {
    let user = create_unique_test_user();

    let resp = register(&app, &user).await;
    assert!(resp.status().is_success());

    let resp = login(&app, &user.email, &user.password_hash).await;
    assert!(resp.status().is_success());

    let tokens: JwtTokenPair = test::read_body_json(resp).await;
    assert!(!tokens.access_token.is_empty());
    assert!(!tokens.refresh_token.is_empty());

    Credentials { user, tokens }
}

#[macro_export]
macro_rules! make_app {
    ($ctx:expr) => {
        actix_web::test::init_service(
            actix_web::App::new()
                .app_data(actix_web::web::Data::new($ctx.state))
                .app_data(actix_multipart::form::tempfile::TempFileConfig::default().directory($ctx.temp_dir.path()))
                .configure(storage_crab::routes::init_routes)
        ).await
    };
}