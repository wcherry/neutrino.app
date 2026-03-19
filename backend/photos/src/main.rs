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
mod faces;
mod learning;
mod persons;
mod photos;
mod schema;
mod suggestions;

use crate::albums::api::{AlbumsApiDoc, AlbumsApiState};
use crate::albums::repository::AlbumsRepository;
use crate::albums::service::AlbumsService;
use crate::faces::api::{FacesApiState, configure_faces};
use crate::faces::repository::FacesRepository;
use crate::faces::service::FacesService;
use crate::persons::api::{PersonsApiState, configure_persons};
use crate::persons::repository::PersonsRepository;
use crate::persons::service::PersonsService;
use crate::learning::api::{LearningApiState, configure_learning};
use crate::learning::repository::LearningRepository;
use crate::learning::service::LearningService;
use crate::suggestions::api::{SuggestionsApiState, configure_suggestions};
use crate::suggestions::repository::SuggestionsRepository;
use crate::suggestions::service::SuggestionsService;
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
    let faces_repo = Arc::new(FacesRepository::new(pool.clone()));
    let persons_repo = Arc::new(PersonsRepository::new(pool.clone()));
    let suggestions_repo = Arc::new(SuggestionsRepository::new(pool.clone()));
    let learning_repo = Arc::new(LearningRepository::new(pool.clone()));

    let photos_service = Arc::new(PhotosService::new(
        photos_repo.clone(),
        drive_client.clone(),
        config.drive_base_url.clone(),
        config.worker_secret.clone(),
    ));
    let albums_service = Arc::new(AlbumsService::new(albums_repo, photos_repo.clone()));
    let faces_service = Arc::new(FacesService::new(faces_repo.clone(), photos_repo));
    let persons_service = Arc::new(PersonsService::new(persons_repo.clone(), suggestions_repo.clone()));
    let suggestions_service = Arc::new(SuggestionsService::new(
        suggestions_repo,
        faces_repo,
        persons_repo.clone(),
        learning_repo.clone(),
    ));
    let learning_service = Arc::new(LearningService::new(
        learning_repo,
        persons_repo,
        suggestions_service.repo.clone(),
    ));

    let photos_state = web::Data::new(PhotosApiState { photos_service: photos_service.clone() });
    let albums_state = web::Data::new(AlbumsApiState { albums_service });
    let faces_state = web::Data::new(FacesApiState { faces_service });
    let persons_state = web::Data::new(PersonsApiState {
        persons_service,
        photos_service,
    });
    let suggestions_state = web::Data::new(SuggestionsApiState { suggestions_service });
    let learning_state = web::Data::new(LearningApiState { learning_service: learning_service.clone() });

    // Background learning loop: periodically process feedback signals and re-evaluate faces.
    let learning_service_bg = learning_service.clone();
    tokio::spawn(async move {
        let interval_secs = std::env::var("REPROCESS_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1800u64);
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
        interval.tick().await; // skip first immediate tick at startup
        loop {
            interval.tick().await;
            if let Err(e) = learning_service_bg.process_all_pending() {
                tracing::error!("Background learning reprocessing error: {:?}", e);
            }
        }
    });

    let token_service_data = web::Data::new(token_service.clone());
    let pool_data = web::Data::new(pool.clone());
    let faces_state_data = faces_state;
    let persons_state_data = persons_state;
    let bind_addr = format!("0.0.0.0:{}", config.port);

    info!("Listening on {}", bind_addr);

    let suggestions_state_data = suggestions_state;

    HttpServer::new(move || {
        let photos_openapi = PhotosApiDoc::openapi();

        App::new()
            .app_data(pool_data.clone())
            .app_data(photos_state.clone())
            .app_data(albums_state.clone())
            .app_data(faces_state_data.clone())
            .app_data(persons_state_data.clone())
            .app_data(suggestions_state_data.clone())
            .app_data(learning_state.clone())
            .app_data(token_service_data.clone())
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(health)
            .service(
                web::scope("/api/v1")
                    .configure(photos::api::configure_photos)
                    .configure(albums::api::configure_albums)
                    .configure(configure_faces)
                    .configure(configure_persons)
                    .configure(configure_suggestions)
                    .configure(configure_learning),
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
