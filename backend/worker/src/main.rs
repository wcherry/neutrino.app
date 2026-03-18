use actix_cors::Cors;
use actix_web::{middleware::Logger, post, web, App, HttpResponse, HttpServer};
use std::sync::Arc;
use tracing::{error, info, warn};

mod config;
mod drive_client;
mod metadata;
mod thumbnail;

use config::Config;
use drive_client::{DriveJobsClient, JobResponse};

// ── Shared state injected into the actix-web callback handler ─────────────────

struct WorkerState {
    drive: DriveJobsClient,
    photos_url: String,
    http: reqwest::Client,
}

// ── Callback endpoint — drive POSTs here to dispatch a job ───────────────────

#[post("/dispatch")]
async fn dispatch(
    state: web::Data<Arc<WorkerState>>,
    job: web::Json<JobResponse>,
) -> HttpResponse {
    let job = job.into_inner();
    info!("Received dispatched job {} (type={})", job.id, job.job_type);

    // Spawn so we return 200 immediately and don't block drive's dispatch call.
    let state = state.get_ref().clone();
    tokio::spawn(async move {
        process_job(&state, job).await;
    });

    HttpResponse::Accepted().finish()
}

// ── Job processing ────────────────────────────────────────────────────────────

async fn process_job(state: &WorkerState, job: JobResponse) {
    let result = match job.job_type.as_str() {
        "thumbnail" => process_thumbnail(state, &job).await,
        other => Err(format!("Unknown job type: {}", other)),
    };

    match result {
        Ok(()) => {
            info!("Job {} completed", job.id);
            if let Err(e) = state.drive.complete_job(&job.id).await {
                error!("Failed to mark job {} complete: {}", job.id, e);
            }
        }
        Err(e) => {
            error!("Job {} failed: {}", job.id, e);
            if let Err(e2) = state.drive.fail_job(&job.id, &e).await {
                error!("Failed to mark job {} failed: {}", job.id, e2);
            }
        }
    }
}

async fn process_thumbnail(state: &WorkerState, job: &JobResponse) -> Result<(), String> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Payload {
        photo_id: String,
        file_id: String,
    }

    let payload: Payload = serde_json::from_value(job.payload.clone())
        .map_err(|e| format!("Invalid payload: {}", e))?;

    // Fetch the image bytes from drive.
    let (bytes, mime_type) = state.drive.get_file_content(&payload.file_id).await?;

    if !mime_type.starts_with("image/") {
        return Err(format!("Not an image (mime_type={})", mime_type));
    }

    // Generate thumbnail and extract metadata on a blocking thread.
    let bytes_arc = std::sync::Arc::new(bytes);
    let bytes_for_thumb = bytes_arc.clone();
    let (thumb, photo_metadata) = tokio::task::spawn_blocking(move || {
        let thumb = thumbnail::generate_jpeg_thumbnail(&bytes_for_thumb);
        let meta = metadata::extract_metadata(&bytes_for_thumb);
        (thumb, meta)
    })
    .await
    .map_err(|e| format!("Thumbnail task panicked: {}", e))?;

    let thumb = thumb?;

    // Upload thumbnail to the photos service.
    let thumb_url = format!(
        "{}/api/v1/photos/{}/thumbnail",
        state.photos_url, payload.photo_id
    );
    let resp = state
        .http
        .put(&thumb_url)
        .header("Content-Type", "image/jpeg")
        .body(thumb)
        .send()
        .await
        .map_err(|e| format!("Thumbnail upload failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Thumbnail upload returned {}: {}", status, body));
    }

    // Upload metadata to the photos service (non-fatal on failure).
    match serde_json::to_string(&photo_metadata) {
        Ok(meta_json) => {
            let meta_url = format!(
                "{}/api/v1/photos/{}/metadata",
                state.photos_url, payload.photo_id
            );
            if let Err(e) = state
                .http
                .put(&meta_url)
                .header("Content-Type", "application/json")
                .body(meta_json)
                .send()
                .await
            {
                tracing::warn!("Metadata upload failed for photo {}: {}", payload.photo_id, e);
            }
        }
        Err(e) => {
            tracing::warn!("Failed to serialize metadata for photo {}: {}", payload.photo_id, e);
        }
    }

    Ok(())
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    });

    std::env::set_var("RUST_LOG", &config.log_level);
    env_logger::init();

    info!("Worker starting on port {}", config.port);
    info!("Drive URL:     {}", config.drive_url);
    info!("Photos URL:    {}", config.photos_url);
    info!("Callback URL:  {}", config.callback_url);

    // Register with drive.
    let worker_id = DriveJobsClient::register(
        &config.drive_url,
        &config.worker_secret,
        &config.callback_url,
    )
    .await
    .unwrap_or_else(|e| {
        error!("Failed to register with drive: {}", e);
        std::process::exit(1);
    });

    info!("Registered with drive as worker {}", worker_id);

    let drive_client = DriveJobsClient::new(
        config.drive_url.clone(),
        config.worker_secret.clone(),
        worker_id.clone(),
    );

    let state = Arc::new(WorkerState {
        drive: DriveJobsClient::new(
            config.drive_url.clone(),
            config.worker_secret.clone(),
            worker_id.clone(),
        ),
        photos_url: config.photos_url.clone(),
        http: reqwest::Client::new(),
    });

    // Pull pending jobs before the HTTP server starts.
    match drive_client.pull_pending(config.initial_batch).await {
        Ok(jobs) if !jobs.is_empty() => {
            info!("Pulled {} pending job(s) from drive", jobs.len());
            for job in jobs {
                let s = state.clone();
                tokio::spawn(async move {
                    process_job(&s, job).await;
                });
            }
        }
        Ok(_) => info!("No pending jobs on startup"),
        Err(e) => warn!("Failed to pull pending jobs: {}", e),
    }

    // Start the HTTP server for drive callbacks.
    let state_data = web::Data::new(state);
    let bind_addr = format!("0.0.0.0:{}", config.port);

    info!("Listening for dispatch callbacks on {}", bind_addr);

    HttpServer::new(move || {
        App::new()
            .app_data(state_data.clone())
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(dispatch)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
