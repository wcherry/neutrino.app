use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{RunQueryDsl, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::{info, error};
use serde_json::json;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod auth;
mod config;
mod schema;
mod common;

use crate::config::Config;
use crate::auth::{
    api::{AuthApiDoc, AuthApiState},
    repository::AuthRepository,
    service::AuthService,
    tokens::TokenService,
};
use shared::init_logging;

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

    init_logging(&config.log_level, config.log_path);

    info!("Starting Neutrino Auth service");
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

    let token_service = Arc::new(TokenService::new_with_expiry(
        config.jwt_secret.clone(),
        config.jwt_access_expiry_secs,
        config.jwt_refresh_expiry_secs,
    ));

    let auth_repo = Arc::new(AuthRepository::new(pool.clone()));
    let auth_service = Arc::new(AuthService::new(auth_repo, token_service.clone()));

    let auth_state = web::Data::new(AuthApiState {
        auth_service: auth_service.clone(),
    });

    let token_service_data = web::Data::new(token_service.clone());
    let pool_data = web::Data::new(pool.clone());
    let bind_addr = format!("0.0.0.0:{}", config.port);

    info!("Listening on {}", bind_addr);

    HttpServer::new(move || {
        let openapi = AuthApiDoc::openapi();

        App::new()
            .app_data(pool_data.clone())
            .app_data(auth_state.clone())
            .app_data(token_service_data.clone())
            .wrap(Cors::permissive())
            .service(health)
            .service(
                web::scope("/api/v1")
                    .configure(auth::api::configure),
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
