use crate::photos::{
    dto::{ListPhotosResponse, PhotoResponse, RegisterPhotoRequest, UpdatePhotoRequest},
    service::PhotosService,
};
use actix_web::{delete, get, patch, post, put, web, HttpResponse};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use shared::auth::AuthenticatedUser;
use shared::ApiError;
use std::sync::Arc;
use utoipa::OpenApi;
use tracing::{debug};

pub struct PhotosApiState {
    pub photos_service: Arc<PhotosService>,
}

#[utoipa::path(
    get,
    path = "/api/v1/photos",
    params(
        ("archivedOnly" = Option<bool>, Query, description = "Include archived photos"),
        ("starredOnly" = Option<bool>, Query, description = "Show only starred photos"),
        ("personIds" = Option<String>, Query, description = "Comma-separated person IDs to filter by (AND logic)"),
    ),
    responses(
        (status = 200, description = "List of photos", body = ListPhotosResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "photos"
)]
#[get("/photos")]
pub async fn list_photos(
    state: web::Data<PhotosApiState>,
    user: AuthenticatedUser,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<web::Json<ListPhotosResponse>, ApiError> {
    let person_ids: Vec<String> = query
        .get("personIds")
        .map(|v| {
            v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let result = if !person_ids.is_empty() {
        state
            .photos_service
            .list_photos_by_person_filter(&user, &person_ids)
            .await?
    } else {
        let include_archived = query
            .get("archivedOnly")
            .map(|v| v == "true")
            .unwrap_or(false);
        let starred_only = query
            .get("starredOnly")
            .map(|v| v == "true")
            .unwrap_or(false);
        state
            .photos_service
            .list_photos(&user, include_archived, starred_only)
            .await?
    };
    Ok(web::Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/photos",
    request_body = RegisterPhotoRequest,
    responses(
        (status = 201, description = "Photo registered", body = PhotoResponse),
        (status = 400, description = "Invalid request"),
    ),
    security(("bearer_auth" = [])),
    tag = "photos"
)]
#[post("/photos")]
pub async fn register_photo(
    state: web::Data<PhotosApiState>,
    user: AuthenticatedUser,
    body: web::Json<RegisterPhotoRequest>,
) -> Result<HttpResponse, ApiError> {
    let photo = state
        .photos_service
        .register_photo(&user, body.into_inner())
        .await?;
    Ok(HttpResponse::Created().json(photo))
}

#[utoipa::path(
    get,
    path = "/api/v1/photos/{id}",
    params(("id" = String, Path, description = "Photo ID")),
    responses(
        (status = 200, description = "Photo metadata", body = PhotoResponse),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "photos"
)]
#[get("/photos/{id}")]
pub async fn get_photo(
    state: web::Data<PhotosApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<web::Json<PhotoResponse>, ApiError> {
    let photo_id = path.into_inner();
    let photo = state.photos_service.get_photo(&user, &photo_id).await?;
    Ok(web::Json(photo))
}

#[utoipa::path(
    patch,
    path = "/api/v1/photos/{id}",
    params(("id" = String, Path, description = "Photo ID")),
    request_body = UpdatePhotoRequest,
    responses(
        (status = 200, description = "Photo updated", body = PhotoResponse),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "photos"
)]
#[patch("/photos/{id}")]
pub async fn update_photo(
    state: web::Data<PhotosApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<UpdatePhotoRequest>,
) -> Result<web::Json<PhotoResponse>, ApiError> {
    let photo_id = path.into_inner();
    let photo = state
        .photos_service
        .update_photo(&user, &photo_id, body.into_inner())
        .await?;
    Ok(web::Json(photo))
}

#[utoipa::path(
    delete,
    path = "/api/v1/photos/{id}",
    params(("id" = String, Path, description = "Photo ID")),
    responses(
        (status = 204, description = "Photo moved to trash"),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "photos"
)]
#[delete("/photos/{id}")]
pub async fn trash_photo(
    state: web::Data<PhotosApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let photo_id = path.into_inner();
    state.photos_service.trash_photo(&user, &photo_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    post,
    path = "/api/v1/photos/{id}/restore",
    params(("id" = String, Path, description = "Photo ID")),
    responses(
        (status = 200, description = "Photo restored", body = PhotoResponse),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "photos"
)]
#[post("/photos/{id}/restore")]
pub async fn restore_photo(
    state: web::Data<PhotosApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<web::Json<PhotoResponse>, ApiError> {
    let photo_id = path.into_inner();
    let photo = state
        .photos_service
        .restore_photo(&user, &photo_id)
        .await?;
    Ok(web::Json(photo))
}

#[utoipa::path(
    get,
    path = "/api/v1/photos/trash",
    responses(
        (status = 200, description = "Trashed photos", body = ListPhotosResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "photos"
)]
#[get("/photos/trash")]
pub async fn list_trash(
    state: web::Data<PhotosApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<ListPhotosResponse>, ApiError> {
    let result = state.photos_service.list_trash(&user).await?;
    Ok(web::Json(result))
}

#[utoipa::path(
    delete,
    path = "/api/v1/photos/trash",
    responses(
        (status = 204, description = "Trash emptied"),
    ),
    security(("bearer_auth" = [])),
    tag = "photos"
)]
#[delete("/photos/trash")]
pub async fn empty_trash(
    state: web::Data<PhotosApiState>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    state.photos_service.empty_trash(&user)?;
    Ok(HttpResponse::NoContent().finish())
}

/// Worker endpoint — stores a generated thumbnail for a photo.
/// Accepts the image bytes as the raw request body.
/// The Content-Type header is used as the thumbnail MIME type.
#[utoipa::path(
    put,
    path = "/api/v1/photos/{id}/thumbnail",
    params(("id" = String, Path, description = "Photo ID")),
    responses(
        (status = 204, description = "Thumbnail saved"),
        (status = 404, description = "Photo not found"),
    ),
    tag = "photos"
)]
#[put("/photos/{id}/thumbnail")]
pub async fn put_thumbnail(
    state: web::Data<PhotosApiState>,
    path: web::Path<String>,
    req: actix_web::HttpRequest,
    body: web::Bytes,
) -> Result<HttpResponse, ApiError> {
    let photo_id = path.into_inner();
    let mime_type = req
        .headers()
        .get("Content-Type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();
    let b64 = BASE64.encode(&body);
    state
        .photos_service
        .save_thumbnail(&photo_id, b64, mime_type)?;
    Ok(HttpResponse::NoContent().finish())
}

/// Returns the raw thumbnail image bytes for a photo.
#[utoipa::path(
    get,
    path = "/api/v1/photos/{id}/thumbnail",
    params(("id" = String, Path, description = "Photo ID")),
    responses(
        (status = 200, description = "Thumbnail image bytes"),
        (status = 404, description = "Photo or thumbnail not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "photos"
)]
#[get("/photos/{id}/thumbnail")]
pub async fn get_thumbnail(
    state: web::Data<PhotosApiState>,
    _user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let photo_id = path.into_inner();
    match state.photos_service.get_thumbnail(&photo_id)? {
        Some((b64, mime_type)) => {
            let bytes = BASE64.decode(&b64).map_err(|_| ApiError::internal("Invalid thumbnail data"))?;
            Ok(HttpResponse::Ok().content_type(mime_type).body(bytes))
        }
        None => Err(ApiError::not_found("Thumbnail not yet available")),
    }
}

/// Worker endpoint — stores extracted image metadata for a photo.
/// Accepts a JSON body. No user auth required (worker-to-service call).
#[utoipa::path(
    put,
    path = "/api/v1/photos/{id}/metadata",
    params(("id" = String, Path, description = "Photo ID")),
    responses(
        (status = 204, description = "Metadata saved"),
        (status = 400, description = "Invalid JSON"),
        (status = 404, description = "Photo not found"),
    ),
    tag = "photos"
)]
#[put("/photos/{id}/metadata")]
pub async fn put_metadata(
    state: web::Data<PhotosApiState>,
    path: web::Path<String>,
    body: web::Bytes,
) -> Result<HttpResponse, ApiError> {
    let photo_id = path.into_inner();
    let metadata = String::from_utf8(body.to_vec())
        .map_err(|_| ApiError::bad_request("Invalid UTF-8 in metadata body"))?;
    state.photos_service.save_metadata(&photo_id, metadata)?;
    Ok(HttpResponse::NoContent().finish())
}

pub fn configure_photos(cfg: &mut web::ServiceConfig) {
    cfg.service(list_trash)
        .service(empty_trash)
        .service(list_photos)
        .service(register_photo)
        .service(get_photo)
        .service(update_photo)
        .service(trash_photo)
        .service(restore_photo)
        .service(put_thumbnail)
        .service(get_thumbnail)
        .service(put_metadata);
}

#[derive(OpenApi)]
#[openapi(
    paths(
        list_photos,
        register_photo,
        get_photo,
        update_photo,
        trash_photo,
        restore_photo,
        list_trash,
        empty_trash,
        put_thumbnail,
        get_thumbnail
    ),
    components(schemas(
        RegisterPhotoRequest,
        UpdatePhotoRequest,
        PhotoResponse,
        ListPhotosResponse,
    )),
    tags((name = "photos", description = "Media library")),
    security(("bearer_auth" = []))
)]
pub struct PhotosApiDoc;
