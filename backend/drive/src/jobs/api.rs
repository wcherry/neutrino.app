use std::sync::Arc;

use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse};
use actix_web::dev::Payload;
use actix_web::FromRequest;
use std::future::{ready, Ready};

use crate::jobs::{
    dto::{
        CreateJobRequest, PendingJobsQuery, RegisterWorkerRequest, RegisterWorkerResponse,
        UpdateJobStatusRequest,
    },
    service::JobsService,
};
use shared::ApiError;

// ── Worker auth ───────────────────────────────────────────────────────────────

/// Newtype wrapper so it can be registered as app data without conflicting with String.
pub struct WorkerSecretData(pub String);

/// Extractor that validates `Authorization: Bearer <WORKER_SECRET>`.
struct WorkerAuth;

impl FromRequest for WorkerAuth {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let expected = req
            .app_data::<web::Data<WorkerSecretData>>()
            .map(|s| s.0.as_str().to_owned());

        let provided = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|s| s.to_owned());

        let ok = match (expected, provided) {
            (Some(exp), Some(prov)) => exp == prov,
            _ => false,
        };

        if ok {
            ready(Ok(WorkerAuth))
        } else {
            ready(Err(ApiError::new(401, "UNAUTHORIZED", "Invalid worker secret")))
        }
    }
}

// ── App state ─────────────────────────────────────────────────────────────────

pub struct JobsApiState {
    pub jobs_service: Arc<JobsService>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// Create a job (called by other services, e.g. photos when a photo is registered).
#[post("/jobs")]
async fn create_job(
    state: web::Data<JobsApiState>,
    _auth: WorkerAuth,
    body: web::Json<CreateJobRequest>,
) -> Result<web::Json<crate::jobs::dto::JobResponse>, ApiError> {
    let resp = state.jobs_service.create_job(body.into_inner())?;
    Ok(web::Json(resp))
}

/// Worker pulls up to `limit` pending jobs on startup and claims them immediately.
#[get("/jobs/pending")]
async fn get_pending_jobs(
    state: web::Data<JobsApiState>,
    _auth: WorkerAuth,
    query: web::Query<PendingJobsQuery>,
    req: HttpRequest,
) -> Result<web::Json<Vec<crate::jobs::dto::JobResponse>>, ApiError> {
    // Worker ID comes from a header so drive knows which worker is claiming.
    let worker_id = req
        .headers()
        .get("X-Worker-Id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_owned();
    let limit = query.limit.unwrap_or(4).min(100);
    let jobs = state.jobs_service.claim_pending_jobs(&worker_id, limit)?;
    Ok(web::Json(jobs))
}

/// Worker reports job completion (status C) or failure (status E).
#[patch("/jobs/{id}/status")]
async fn update_job_status(
    state: web::Data<JobsApiState>,
    _auth: WorkerAuth,
    path: web::Path<String>,
    body: web::Json<UpdateJobStatusRequest>,
) -> Result<HttpResponse, ApiError> {
    state
        .jobs_service
        .update_job_status(&path.into_inner(), body.into_inner())?;
    Ok(HttpResponse::NoContent().finish())
}

/// Worker fetches raw file bytes to process (e.g. generate a thumbnail).
#[get("/jobs/file-content/{file_id}")]
async fn get_file_content(
    state: web::Data<JobsApiState>,
    _auth: WorkerAuth,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let (bytes, mime_type) = state
        .jobs_service
        .get_file_content(&path.into_inner())
        .await?;
    Ok(HttpResponse::Ok()
        .content_type(mime_type)
        .body(bytes))
}

/// Worker registers itself and provides a callback URL for job dispatch.
#[post("/jobs/workers")]
async fn register_worker(
    state: web::Data<JobsApiState>,
    _auth: WorkerAuth,
    body: web::Json<RegisterWorkerRequest>,
) -> Result<web::Json<RegisterWorkerResponse>, ApiError> {
    let worker_id = state
        .jobs_service
        .register_worker(&body.callback_url)?;
    Ok(web::Json(RegisterWorkerResponse { worker_id }))
}

/// Worker deregisters itself (clean shutdown).
#[delete("/jobs/workers/{id}")]
async fn deregister_worker(
    state: web::Data<JobsApiState>,
    _auth: WorkerAuth,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    state
        .jobs_service
        .deregister_worker(&path.into_inner())?;
    Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create_job)
        .service(get_pending_jobs)
        .service(update_job_status)
        .service(get_file_content)
        .service(register_worker)
        .service(deregister_worker);
}
