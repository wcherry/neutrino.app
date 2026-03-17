use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{RunQueryDsl, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::{error, info};
use serde_json::json;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Config as SwaggerConfig};
use actix_cors::Cors;
mod access_requests;
mod config;
mod filesystem;
mod irm;
mod jobs;
mod permissions;
mod schema;
mod common;
mod sharing;
mod storage;
mod workspace;

use crate::access_requests::{
    api::{AccessRequestsApiDoc, AccessRequestsApiState},
    repository::AccessRequestsRepository,
    service::AccessRequestsService,
};
use crate::config::Config;
use crate::filesystem::{
    api::{FilesystemApiDoc, FilesystemApiState},
    repository::FilesystemRepository,
    service::FilesystemService,
};
use crate::irm::{
    api::{IrmApiDoc, IrmApiState},
    repository::IrmRepository,
    service::IrmService,
};
use crate::jobs::{
    api::{JobsApiState, WorkerSecretData},
    repository::JobsRepository,
    service::JobsService,
};
use crate::permissions::{
    api::{PermissionsApiDoc, PermissionsApiState},
    repository::PermissionsRepository,
    service::PermissionsService,
};
use crate::sharing::{
    api::{SharingApiDoc, SharingApiState},
    repository::SharingRepository,
    service::SharingService,
};
use crate::common::TokenService;
use crate::storage::{
    api::{StorageApiDoc, StorageApiState},
    repository::StorageRepository,
    service::StorageService,
    store::LocalFileStore,
};
use crate::workspace::{
    api::{WorkspaceApiDoc, WorkspaceApiState},
    repository::WorkspaceRepository,
    service::WorkspaceService,
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

    // Workspace settings (no dependencies — created first so others can use it)
    let workspace_repo = Arc::new(WorkspaceRepository::new(pool.clone()));
    let workspace_service = Arc::new(WorkspaceService::new(workspace_repo));
    let workspace_state = web::Data::new(WorkspaceApiState {
        workspace_service: workspace_service.clone(),
    });

    // Permissions setup (shared across storage and filesystem)
    let permissions_repo = Arc::new(PermissionsRepository::new(pool.clone()));
    let permissions_service = Arc::new(PermissionsService::new(
        permissions_repo.clone(),
        workspace_service.clone(),
    ));
    let permissions_state = web::Data::new(PermissionsApiState {
        permissions_service: permissions_service.clone(),
    });

    // IRM setup
    let irm_repo = Arc::new(IrmRepository::new(pool.clone()));
    let irm_service = Arc::new(IrmService::new(irm_repo, permissions_service.clone()));
    let irm_state = web::Data::new(IrmApiState {
        irm_service: irm_service.clone(),
    });

    let storage_repo = Arc::new(StorageRepository::new(pool.clone()));
    let storage_service = Arc::new(StorageService::new(
        storage_repo,
        file_store.clone(),
        permissions_service.clone(),
    ));

    let storage_state = web::Data::new(StorageApiState {
        storage_service: storage_service.clone(),
        irm_service: irm_service.clone(),
        permissions_service: permissions_service.clone(),
    });

    // Filesystem setup
    let fs_repo = Arc::new(FilesystemRepository::new(pool.clone()));
    let fs_service = Arc::new(FilesystemService::new(
        fs_repo.clone(),
        file_store,
        permissions_service.clone(),
    ));
    let fs_state = web::Data::new(FilesystemApiState {
        filesystem_service: fs_service,
        filesystem_repo: fs_repo,
        permissions_repo: permissions_repo.clone(),
    });

    // Sharing setup
    let sharing_repo = Arc::new(SharingRepository::new(pool.clone()));
    let sharing_service = Arc::new(SharingService::new(
        sharing_repo,
        permissions_service.clone(),
        workspace_service,
    ));
    let sharing_state = web::Data::new(SharingApiState {
        sharing_service,
        irm_service,
    });

    // Access requests setup
    let access_requests_repo = Arc::new(AccessRequestsRepository::new(pool.clone()));
    let access_requests_service = Arc::new(AccessRequestsService::new(
        access_requests_repo,
        permissions_repo,
        permissions_service.clone(),
    ));
    let access_requests_state = web::Data::new(AccessRequestsApiState {
        service: access_requests_service,
    });

    // Jobs setup
    let jobs_repo = Arc::new(JobsRepository::new(pool.clone()));
    let jobs_service = Arc::new(JobsService::new(jobs_repo, config.storage_path.clone(), config.jobs_per_worker));
    let jobs_state = web::Data::new(JobsApiState {
        jobs_service: jobs_service.clone(),
    });
    let worker_secret_data = web::Data::new(WorkerSecretData(config.worker_secret.clone()));

    // Background task: reset timed-out jobs and dispatch ready jobs to workers.
    let jobs_bg = jobs_service.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            jobs_bg.process_background_tasks().await;
        }
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
        openapi.merge(PermissionsApiDoc::openapi());
        openapi.merge(SharingApiDoc::openapi());
        openapi.merge(AccessRequestsApiDoc::openapi());
        openapi.merge(IrmApiDoc::openapi());
        openapi.merge(WorkspaceApiDoc::openapi());
        let config = SwaggerConfig::new(vec![
            "http://localhost:8881/api-docs/openapi.json",
            "http://localhost:8882/api-docs/openapi.json"
        ]);

        App::new()
            .app_data(web::PayloadConfig::new(max_upload_bytes))
            .app_data(pool_data.clone())
            .app_data(storage_state.clone())
            .app_data(fs_state.clone())
            .app_data(permissions_state.clone())
            .app_data(sharing_state.clone())
            .app_data(access_requests_state.clone())
            .app_data(irm_state.clone())
            .app_data(workspace_state.clone())
            .app_data(jobs_state.clone())
            .app_data(worker_secret_data.clone())
            .app_data(token_service_data.clone())
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(health)
            .service(
                web::scope("/api/v1/drive")
                    .configure(storage::api::configure)
                    .configure(filesystem::api::configure)
                    .configure(permissions::api::configure)
                    .configure(sharing::api::configure_drive)
                    .configure(access_requests::api::configure)
                    .configure(irm::api::configure),
            )
            .service(
                web::scope("/api/v1")
                    .configure(sharing::api::configure_public)
                    .configure(jobs::api::configure),
            )
            .service(
                web::scope("/api/v1/admin")
                    .configure(workspace::api::configure),
            )
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi)
                    .config(config.clone()),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}
