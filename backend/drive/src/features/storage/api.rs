use crate::features::shared::{ApiError, AuthenticatedUser, ListQuery};
use crate::features::storage::{
    dto::{FileMetadataResponse, FileOrderField, ListFilesResponse, QuotaResponse},
    service::StorageService,
};
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use utoipa::OpenApi;
use uuid::Uuid;

pub struct StorageApiState {
    pub storage_service: Arc<StorageService>,
}

#[utoipa::path(
    post,
    path = "/api/v1/storage/files",
    responses(
        (status = 201, description = "File uploaded successfully", body = FileMetadataResponse),
        (status = 400, description = "No file provided or invalid multipart data"),
        (status = 413, description = "Storage quota exceeded"),
        (status = 429, description = "Daily upload limit exceeded"),
    ),
    security(("bearer_auth" = [])),
    tag = "storage"
)]
#[post("/files")]
pub async fn upload_file(
    state: web::Data<StorageApiState>,
    user: AuthenticatedUser,
    mut payload: Multipart,
) -> Result<web::Json<FileMetadataResponse>, ApiError> {
    while let Some(field) = payload.next().await {
        let mut field = field.map_err(|e| {
            log::error!("Multipart error: {:?}", e);
            ApiError::bad_request("Invalid multipart data")
        })?;

        let file_name = field
            .content_disposition()
            .and_then(|cd| cd.get_filename())
            .unwrap_or("untitled")
            .to_string();

        let mime_type = field
            .content_type()
            .map(|m| m.to_string())
            .unwrap_or_else(|| {
                mime_guess::from_path(&file_name)
                    .first_or_octet_stream()
                    .to_string()
            });

        let temp_id = Uuid::new_v4().to_string();
        state
            .storage_service
            .store()
            .ensure_user_dir(&user.user_id)
            .map_err(ApiError::internal)?;

        let temp_path = state
            .storage_service
            .store()
            .temp_path(&user.user_id, &temp_id);

        let mut file = tokio::fs::File::create(&temp_path).await.map_err(|e| {
            log::error!("Failed to create temp file: {:?}", e);
            ApiError::internal("Failed to initialize upload")
        })?;

        let mut size: i64 = 0;
        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| {
                log::error!("Chunk read error: {:?}", e);
                ApiError::bad_request("Upload interrupted")
            })?;
            size += data.len() as i64;
            file.write_all(&data).await.map_err(|e| {
                log::error!("Write error: {:?}", e);
                ApiError::internal("Failed to write upload data")
            })?;
        }

        file.flush().await.map_err(|e| {
            log::error!("Flush error: {:?}", e);
            ApiError::internal("Failed to finalize upload")
        })?;
        drop(file);

        let response = state
            .storage_service
            .finalize_upload(&user.user_id, &temp_path, &file_name, &mime_type, size)
            .inspect_err(|_| {
                let _ = std::fs::remove_file(&temp_path);
            })?;

        return Ok(web::Json(response));
    }

    Err(ApiError::bad_request("No file provided in multipart body"))
}

#[utoipa::path(
    get,
    path = "/api/v1/storage/files",
    params(
        ("limit" = Option<i64>, Query, description = "Max results per page (default 50)"),
        ("offset" = Option<i64>, Query, description = "Pagination offset"),
        ("orderBy" = Option<FileOrderField>, Query, description = "Sort field"),
        ("direction" = Option<String>, Query, description = "asc or desc"),
    ),
    responses(
        (status = 200, description = "List of files", body = ListFilesResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "storage"
)]
#[get("/files")]
pub async fn list_files(
    state: web::Data<StorageApiState>,
    user: AuthenticatedUser,
    query: web::Query<ListQuery<FileOrderField>>,
) -> Result<web::Json<ListFilesResponse>, ApiError> {
    let response = state
        .storage_service
        .list_files(&user.user_id, &query.into_inner())?;
    Ok(web::Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/storage/files/{id}/metadata",
    params(("id" = String, Path, description = "File ID")),
    responses(
        (status = 200, description = "File metadata", body = FileMetadataResponse),
        (status = 404, description = "File not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "storage"
)]
#[get("/files/{id}/metadata")]
pub async fn get_file_metadata(
    state: web::Data<StorageApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<web::Json<FileMetadataResponse>, ApiError> {
    let file_id = path.into_inner();
    let metadata = state
        .storage_service
        .get_file_metadata(&user.user_id, &file_id)?;
    Ok(web::Json(metadata))
}

#[utoipa::path(
    get,
    path = "/api/v1/storage/files/{id}",
    params(("id" = String, Path, description = "File ID")),
    responses(
        (status = 200, description = "File content (supports Range requests for resume)"),
        (status = 404, description = "File not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "storage"
)]
#[get("/files/{id}")]
pub async fn download_file(
    state: web::Data<StorageApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let file_id = path.into_inner();
    let (file_path, mime_type, file_name) = state
        .storage_service
        .resolve_file_path(&user.user_id, &file_id)?;

    let content_type: mime::Mime = mime_type
        .parse()
        .unwrap_or(mime::APPLICATION_OCTET_STREAM);

    let disposition = actix_web::http::header::ContentDisposition {
        disposition: actix_web::http::header::DispositionType::Attachment,
        parameters: vec![actix_web::http::header::DispositionParam::Filename(
            file_name,
        )],
    };

    let named_file = NamedFile::open(&file_path)
        .map_err(|e| {
            log::error!("Failed to open file {:?}: {:?}", file_path, e);
            ApiError::internal("Failed to serve file")
        })?
        .set_content_type(content_type)
        .set_content_disposition(disposition);

    Ok(named_file.into_response(&req))
}

#[utoipa::path(
    get,
    path = "/api/v1/storage/quota",
    responses(
        (status = 200, description = "Current user quota usage", body = QuotaResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "storage"
)]
#[get("/quota")]
pub async fn get_quota(
    state: web::Data<StorageApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<QuotaResponse>, ApiError> {
    let quota = state.storage_service.get_quota(&user.user_id)?;
    Ok(web::Json(quota))
}

pub fn configure(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/storage")
            .service(upload_file)
            .service(list_files)
            .service(get_file_metadata)
            .service(download_file)
            .service(get_quota),
    );
}

#[derive(OpenApi)]
#[openapi(
    paths(upload_file, list_files, get_file_metadata, download_file, get_quota),
    components(schemas(
        FileMetadataResponse,
        FileOrderField,
        ListFilesResponse,
        QuotaResponse
    )),
    tags((name = "storage", description = "File storage endpoints")),
    modifiers(&SecurityAddon)
)]
pub struct StorageApiDoc;

struct SecurityAddon;
impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
