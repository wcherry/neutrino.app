use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{RunQueryDsl, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::{error, info};
use serde_json::json;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod filesystem;
mod schema;
mod shared;
mod storage;

use crate::config::Config;
use crate::filesystem::{
    api::{FilesystemApiDoc, FilesystemApiState},
    repository::FilesystemRepository,
    service::FilesystemService,
};
use crate::shared::TokenService;
use crate::storage::{
    api::{StorageApiDoc, StorageApiState},
    repository::StorageRepository,
    service::StorageService,
    store::LocalFileStore,
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

type DbPool = Pool<ConnectionManager<SqliteConnection>>;

fn create_db_pool(database_url: &str) -> Result<DbPool, String> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .map_err(|e| format!("Failed to create DB pool: {}", e))
}

fn run_migrations(pool: &DbPool) -> Result<(), String> {
    let mut conn = pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| format!("Failed to run migrations: {}", e))?;
    Ok(())
}

#[get("/health")]
async fn health(pool: web::Data<DbPool>) -> impl Responder {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Health check DB connection error: {:?}", e);
            return HttpResponse::ServiceUnavailable().json(json!({
                "error": {
                    "code": "DB_UNAVAILABLE",
                    "message": "Database connection unavailable"
                }
            }));
        }
    };

    match diesel::sql_query("SELECT 1").execute(&mut conn) {
        Ok(_) => HttpResponse::Ok().json(json!({"status": "ok"})),
        Err(e) => {
            error!("Health check DB query error: {:?}", e);
            HttpResponse::ServiceUnavailable().json(json!({
                "error": {
                    "code": "DB_UNHEALTHY",
                    "message": "Database health check failed"
                }
            }))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    });

    std::env::set_var("RUST_LOG", &config.log_level);
    env_logger::init();

    info!("Starting Neutrino Drive service");
    info!("Connecting to database: {}", config.database_url);

    let pool = create_db_pool(&config.database_url).unwrap_or_else(|e| {
        error!("{}", e);
        std::process::exit(1);
    });

    run_migrations(&pool).unwrap_or_else(|e| {
        error!("{}", e);
        std::process::exit(1);
    });

    info!("Database migrations applied");

    let token_service = Arc::new(TokenService::new(config.jwt_secret.clone()));

    // Storage setup
    let file_store = Arc::new(
        LocalFileStore::new(&config.storage_path).unwrap_or_else(|e| {
            error!("{}", e);
            std::process::exit(1);
        }),
    );

    let storage_repo = Arc::new(StorageRepository::new(pool.clone()));
    let storage_service = Arc::new(StorageService::new(storage_repo, file_store.clone()));

    let storage_state = web::Data::new(StorageApiState {
        storage_service: storage_service.clone(),
    });

    // Filesystem setup
    let fs_repo = Arc::new(FilesystemRepository::new(pool.clone()));
    let fs_service = Arc::new(FilesystemService::new(fs_repo, file_store));
    let fs_state = web::Data::new(FilesystemApiState {
        filesystem_service: fs_service,
    });

    let token_service_data = web::Data::new(token_service.clone());

    let pool_data = web::Data::new(pool.clone());
    let bind_addr = format!("0.0.0.0:{}", config.port);
    let max_upload_bytes = config.max_upload_bytes as usize;

    info!("Storage path: {}", config.storage_path);
    info!("Max upload size: {} bytes", config.max_upload_bytes);
    info!("Listening on {}", bind_addr);

    HttpServer::new(move || {
        let mut openapi = StorageApiDoc::openapi();
        openapi.merge(FilesystemApiDoc::openapi());

        App::new()
            .app_data(web::PayloadConfig::new(max_upload_bytes))
            .app_data(pool_data.clone())
            .app_data(storage_state.clone())
            .app_data(fs_state.clone())
            .app_data(token_service_data.clone())
            .wrap(Logger::default())
            .service(health)
            .service(
                web::scope("/api/v1/drive")
                    .configure(storage::api::configure)
                    .configure(filesystem::api::configure),
            )
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}
