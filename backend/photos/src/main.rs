use actix_cors::Cors;
use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{RunQueryDsl, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use serde_json::json;
use shared::auth::tokens::TokenService;
use crate::config::Config;
use shared::drive_client::DriveClient;
use std::sync::Arc;
use tracing::{error, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod albums;
mod config;
mod photos;
mod schema;

use crate::albums::api::{AlbumsApiDoc, AlbumsApiState};
use crate::albums::repository::AlbumsRepository;
use crate::albums::service::AlbumsService;
use crate::photos::api::{PhotosApiDoc, PhotosApiState};
use crate::photos::repository::PhotosRepository;
use crate::photos::service::PhotosService;

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

    info!("Starting Neutrino Photos service");
    info!("Connecting to database: {}", config.database_url);

    let pool = create_db_pool(&config.database_url).unwrap_or_else(|e| {
        error!("{}", e);
        std::process::exit(1);
    });

    run_migrations(&pool).unwrap_or_else(|e| {
        error!("{}", e);
        std::process::exit(1);
    });

    let token_service = Arc::new(TokenService::new(config.jwt_secret.clone()));
    let drive_client = Arc::new(DriveClient::new(config.drive_base_url.clone()));

    let photos_repo = Arc::new(PhotosRepository::new(pool.clone()));
    let albums_repo = Arc::new(AlbumsRepository::new(pool.clone()));

    let photos_service = Arc::new(PhotosService::new(
        photos_repo.clone(),
        drive_client.clone(),
        config.drive_base_url.clone(),
        config.worker_secret.clone(),
    ));
    let albums_service = Arc::new(AlbumsService::new(albums_repo, photos_repo));

    let photos_state = web::Data::new(PhotosApiState { photos_service });
    let albums_state = web::Data::new(AlbumsApiState { albums_service });

    let token_service_data = web::Data::new(token_service.clone());
    let pool_data = web::Data::new(pool.clone());
    let bind_addr = format!("0.0.0.0:{}", config.port);

    info!("Listening on {}", bind_addr);

    HttpServer::new(move || {
        let photos_openapi = PhotosApiDoc::openapi();

        App::new()
            .app_data(pool_data.clone())
            .app_data(photos_state.clone())
            .app_data(albums_state.clone())
            .app_data(token_service_data.clone())
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(health)
            .service(
                web::scope("/api/v1")
                    .configure(photos::api::configure_photos)
                    .configure(albums::api::configure_albums),
            )
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", photos_openapi),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}
