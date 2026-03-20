use actix_cors::Cors;
use actix_web::{middleware::Logger, post, web, App, HttpRequest, HttpResponse, HttpServer};
use ort::session::Session;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

mod config;
mod drive_client;
mod face_cluster;
mod face_detect;
mod face_recognize;
mod metadata;
mod thumbnail;

use config::Config;
use drive_client::{DriveJobsClient, JobResponse};

// ── Shared state injected into the actix-web callback handler ─────────────────

struct WorkerState {
    drive: DriveJobsClient,
    photos_url: String,
    /// Loaded InsightFace SCRFD session, or None if no model path was configured.
    face_session: Option<Arc<Mutex<Session>>>,
    /// Loaded ArcFace recognition session, or None if no model path was configured.
    recognition_session: Option<Arc<Mutex<Session>>>,
    face_cluster_eps: f32,
    face_cluster_min_samples: usize,
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

// ── Admin endpoint — enqueue cluster jobs for all users ───────────────────────

#[post("/admin/cluster-all")]
async fn cluster_all(
    state: web::Data<Arc<WorkerState>>,
    req: HttpRequest,
) -> HttpResponse {
    // Require the worker secret in the Authorization header.
    let expected = format!("Bearer {}", state.drive.worker_secret());
    let auth = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if auth != expected {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}));
    }

    let users_url = format!("{}/api/v1/internal/users-with-faces", state.photos_url);
    let user_ids: Vec<String> = match state.http.get(&users_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Body { user_ids: Vec<String> }
            match resp.json::<Body>().await {
                Ok(b) => b.user_ids,
                Err(e) => {
                    error!("Failed to parse users-with-faces response: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(serde_json::json!({"error": "Failed to parse users response"}));
                }
            }
        }
        Ok(resp) => {
            error!("users-with-faces returned {}", resp.status());
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Photos service error"}));
        }
        Err(e) => {
            error!("Failed to reach photos service: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Photos service unreachable"}));
        }
    };

    let count = user_ids.len();
    info!("cluster-all: enqueuing face_cluster for {} user(s)", count);

    for user_id in &user_ids {
        if let Err(e) = state
            .drive
            .enqueue_job(
                "face_cluster",
                serde_json::json!({ "userId": user_id }),
                120,
                state.drive.worker_secret(),
            )
            .await
        {
            warn!("cluster-all: failed to enqueue job for user {}: {}", user_id, e);
        }
    }

    HttpResponse::Ok().json(serde_json::json!({ "queued": count }))
}

// ── Job processing ────────────────────────────────────────────────────────────

async fn process_job(state: &WorkerState, job: JobResponse) {
    let result = match job.job_type.as_str() {
        "thumbnail" => process_thumbnail(state, &job).await,
        "face_detect" => {
            face_detect::process_face_detect(
                face_detect::FaceDetectDeps {
                    drive: &state.drive,
                    photos_url: &state.photos_url,
                    face_session: state.face_session.clone(),
                    recognition_session: state.recognition_session.clone(),
                    http: &state.http,
                },
                &job,
            )
            .await
        }
        "face_cluster" => {
            face_cluster::process_face_cluster(
                face_cluster::FaceClusterDeps {
                    photos_url: &state.photos_url,
                    http: &state.http,
                    eps: state.face_cluster_eps,
                    min_samples: state.face_cluster_min_samples,
                },
                &job,
            )
            .await
        }
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
        file_id: String,
        // photo_id is kept for backward compatibility with jobs already in the queue
        photo_id: Option<String>,
    }

    let payload: Payload = serde_json::from_value(job.payload.clone())
        .map_err(|e| format!("Invalid payload: {}", e))?;

    // Fetch the file bytes from drive.
    let (bytes, mime_type) = state.drive.get_file_content(&payload.file_id).await?;

    // Generate thumbnail (and optionally EXIF metadata for images) on a blocking thread.
    let is_image = mime_type.starts_with("image/");
    let mime_type_clone = mime_type.clone();
    let bytes_arc = std::sync::Arc::new(bytes);
    let bytes_for_thumb = bytes_arc.clone();
    let (thumb, photo_metadata) = tokio::task::spawn_blocking(move || {
        let thumb = thumbnail::generate_thumbnail_for_type(&bytes_for_thumb, &mime_type_clone);
        let meta: Option<metadata::PhotoMetadata> = if is_image {
            Some(metadata::extract_metadata(&bytes_for_thumb))
        } else {
            None
        };
        (thumb, meta)
    })
    .await
    .map_err(|e| format!("Thumbnail task panicked: {}", e))?;

    let thumb = thumb?;

    // Upload thumbnail to the drive service (stored as cover_thumbnail on the file).
    let thumb_url = format!(
        "{}/api/v1/jobs/files/{}/thumbnail",
        state.drive.base_url(), payload.file_id
    );
    let resp = state
        .http
        .put(&thumb_url)
        .header("Authorization", format!("Bearer {}", state.drive.worker_secret()))
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

    // Upload metadata to the photos service if we have a photo_id and image metadata (non-fatal).
    if let (Some(photo_id), Some(meta)) = (payload.photo_id, photo_metadata) {
        match serde_json::to_string(&meta) {
            Ok(meta_json) => {
                let meta_url = format!(
                    "{}/api/v1/photos/{}/metadata",
                    state.photos_url, photo_id
                );
                if let Err(e) = state
                    .http
                    .put(&meta_url)
                    .header("Content-Type", "application/json")
                    .body(meta_json)
                    .send()
                    .await
                {
                    tracing::warn!("Metadata upload failed for photo {}: {}", photo_id, e);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to serialize metadata for photo {}: {}", photo_id, e);
            }
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
    info!("Drive URL:              {}", config.drive_url);
    info!("Photos URL:             {}", config.photos_url);
    info!(
        "Face detect model:      {}",
        if config.face_detect_model_path.is_empty() {
            "<not configured>"
        } else {
            &config.face_detect_model_path
        }
    );
    info!(
        "Face recognition model: {}",
        if config.face_recognition_model_path.is_empty() {
            "<not configured>"
        } else {
            &config.face_recognition_model_path
        }
    );
    info!("Cluster eps:            {}", config.face_cluster_eps);
    info!("Cluster min_samples:    {}", config.face_cluster_min_samples);
    info!("Callback URL:           {}", config.callback_url);

    // Load the InsightFace SCRFD ONNX session if a model path was provided.
    let face_session: Option<Arc<Mutex<Session>>> = if config.face_detect_model_path.is_empty() {
        warn!("FACE_DETECT_MODEL_PATH is not set — face_detect jobs will be skipped");
        None
    } else {
        match Session::builder()
            .map_err(|e| format!("ort session builder failed: {}", e))
            .and_then(|b| {
                b.commit_from_file(&config.face_detect_model_path)
                    .map_err(|e| format!(
                        "Failed to load face detection model from '{}': {}",
                        config.face_detect_model_path, e
                    ))
            }) {
            Ok(session) => {
                info!("Loaded face detection model from '{}'", config.face_detect_model_path);
                Some(Arc::new(Mutex::new(session)))
            }
            Err(e) => {
                error!("{}", e);
                std::process::exit(1);
            }
        }
    };

    // Load the ArcFace recognition ONNX session if a model path was provided.
    let recognition_session: Option<Arc<Mutex<Session>>> =
        if config.face_recognition_model_path.is_empty() {
            warn!("FACE_RECOGNITION_MODEL_PATH is not set — face embeddings will be skipped");
            None
        } else {
            match Session::builder()
                .map_err(|e| format!("ort session builder failed: {}", e))
                .and_then(|b| {
                    b.commit_from_file(&config.face_recognition_model_path)
                        .map_err(|e| format!(
                            "Failed to load recognition model from '{}': {}",
                            config.face_recognition_model_path, e
                        ))
                }) {
                Ok(session) => {
                    info!(
                        "Loaded face recognition model from '{}'",
                        config.face_recognition_model_path
                    );
                    Some(Arc::new(Mutex::new(session)))
                }
                Err(e) => {
                    error!("{}", e);
                    std::process::exit(1);
                }
            }
        };

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
        face_session,
        recognition_session,
        face_cluster_eps: config.face_cluster_eps,
        face_cluster_min_samples: config.face_cluster_min_samples,
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
            .service(cluster_all)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
