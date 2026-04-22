use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::{postgres::Postgres, redis::Redis};
use storage_crab::{AppState, create_db_pool, create_redis_pool};
use tempfile::TempDir;

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

    TestContext {
        state: AppState {
            secret: "test-secret".into(),
            db_pool,
            redis_pool,
        },
        temp_dir,
        _pg: pg,
        _redis: redis,
    }
}

#[macro_export]
macro_rules! make_app {
    ($ctx:expr) => {
        actix_web::test::init_service(
            actix_web::App::new()
                .app_data(web::Data::new($ctx.state))
                .app_data(actix_multipart::form::tempfile::TempFileConfig::default().directory($ctx.temp_dir.path()))
                .configure(init_routes)
        ).await
    };
}