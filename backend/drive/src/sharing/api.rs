use crate::sharing::{
    dto::{
        ResolvedShareLinkResponse, ShareLinkResponse, UpdateShareLinkRequest,
        UpsertShareLinkRequest,
    },
    service::SharingService,
};
use crate::shared::{ApiError, AuthenticatedUser};
use actix_web::{delete, get, patch, put, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

pub struct SharingApiState {
    pub sharing_service: Arc<SharingService>,
}

// ── File share link endpoints ─────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/drive/files/{file_id}/share-link",
    params(("file_id" = String, Path, description = "File ID")),
    responses(
        (status = 200, description = "Share link", body = ShareLinkResponse),
        (status = 404, description = "No share link exists"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = [])),
    tag = "sharing"
)]
#[get("/files/{file_id}/share-link")]
pub async fn get_file_share_link(
    state: web::Data<SharingApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let file_id = path.into_inner();
    match state
        .sharing_service
        .get_share_link(&user.user_id, "file", &file_id)?
    {
        Some(link) => Ok(HttpResponse::Ok().json(link)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": { "code": "NOT_FOUND", "message": "No share link exists for this file" }
        }))),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/drive/files/{file_id}/share-link",
    params(("file_id" = String, Path, description = "File ID")),
    request_body = UpsertShareLinkRequest,
    responses(
        (status = 200, description = "Share link created or replaced", body = ShareLinkResponse),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = [])),
    tag = "sharing"
)]
#[put("/files/{file_id}/share-link")]
pub async fn upsert_file_share_link(
    state: web::Data<SharingApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<UpsertShareLinkRequest>,
) -> Result<web::Json<ShareLinkResponse>, ApiError> {
    let file_id = path.into_inner();
    let link = state.sharing_service.upsert_share_link(
        &user.user_id,
        "file",
        &file_id,
        body.into_inner(),
    )?;
    Ok(web::Json(link))
}

#[utoipa::path(
    patch,
    path = "/api/v1/drive/files/{file_id}/share-link",
    params(("file_id" = String, Path, description = "File ID")),
    request_body = UpdateShareLinkRequest,
    responses(
        (status = 200, description = "Share link updated", body = ShareLinkResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Share link not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "sharing"
)]
#[patch("/files/{file_id}/share-link")]
pub async fn update_file_share_link(
    state: web::Data<SharingApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<UpdateShareLinkRequest>,
) -> Result<web::Json<ShareLinkResponse>, ApiError> {
    let file_id = path.into_inner();
    let link = state.sharing_service.update_share_link(
        &user.user_id,
        "file",
        &file_id,
        body.into_inner(),
    )?;
    Ok(web::Json(link))
}

#[utoipa::path(
    delete,
    path = "/api/v1/drive/files/{file_id}/share-link",
    params(("file_id" = String, Path, description = "File ID")),
    responses(
        (status = 204, description = "Share link removed"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Share link not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "sharing"
)]
#[delete("/files/{file_id}/share-link")]
pub async fn delete_file_share_link(
    state: web::Data<SharingApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let file_id = path.into_inner();
    state
        .sharing_service
        .delete_share_link(&user.user_id, "file", &file_id)?;
    Ok(HttpResponse::NoContent().finish())
}

// ── Folder share link endpoints ───────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/drive/folders/{folder_id}/share-link",
    params(("folder_id" = String, Path, description = "Folder ID")),
    responses(
        (status = 200, description = "Share link", body = ShareLinkResponse),
        (status = 404, description = "No share link exists"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = [])),
    tag = "sharing"
)]
#[get("/folders/{folder_id}/share-link")]
pub async fn get_folder_share_link(
    state: web::Data<SharingApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let folder_id = path.into_inner();
    match state
        .sharing_service
        .get_share_link(&user.user_id, "folder", &folder_id)?
    {
        Some(link) => Ok(HttpResponse::Ok().json(link)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": { "code": "NOT_FOUND", "message": "No share link exists for this folder" }
        }))),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/drive/folders/{folder_id}/share-link",
    params(("folder_id" = String, Path, description = "Folder ID")),
    request_body = UpsertShareLinkRequest,
    responses(
        (status = 200, description = "Share link created or replaced", body = ShareLinkResponse),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = [])),
    tag = "sharing"
)]
#[put("/folders/{folder_id}/share-link")]
pub async fn upsert_folder_share_link(
    state: web::Data<SharingApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<UpsertShareLinkRequest>,
) -> Result<web::Json<ShareLinkResponse>, ApiError> {
    let folder_id = path.into_inner();
    let link = state.sharing_service.upsert_share_link(
        &user.user_id,
        "folder",
        &folder_id,
        body.into_inner(),
    )?;
    Ok(web::Json(link))
}

#[utoipa::path(
    patch,
    path = "/api/v1/drive/folders/{folder_id}/share-link",
    params(("folder_id" = String, Path, description = "Folder ID")),
    request_body = UpdateShareLinkRequest,
    responses(
        (status = 200, description = "Share link updated", body = ShareLinkResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Share link not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "sharing"
)]
#[patch("/folders/{folder_id}/share-link")]
pub async fn update_folder_share_link(
    state: web::Data<SharingApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<UpdateShareLinkRequest>,
) -> Result<web::Json<ShareLinkResponse>, ApiError> {
    let folder_id = path.into_inner();
    let link = state.sharing_service.update_share_link(
        &user.user_id,
        "folder",
        &folder_id,
        body.into_inner(),
    )?;
    Ok(web::Json(link))
}

#[utoipa::path(
    delete,
    path = "/api/v1/drive/folders/{folder_id}/share-link",
    params(("folder_id" = String, Path, description = "Folder ID")),
    responses(
        (status = 204, description = "Share link removed"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Share link not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "sharing"
)]
#[delete("/folders/{folder_id}/share-link")]
pub async fn delete_folder_share_link(
    state: web::Data<SharingApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let folder_id = path.into_inner();
    state
        .sharing_service
        .delete_share_link(&user.user_id, "folder", &folder_id)?;
    Ok(HttpResponse::NoContent().finish())
}

// ── Public resolution endpoint ────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/share/{token}",
    params(("token" = String, Path, description = "Share link token")),
    responses(
        (status = 200, description = "Resolved share link", body = ResolvedShareLinkResponse),
        (status = 404, description = "Share link not found or disabled"),
        (status = 410, description = "Share link has expired"),
    ),
    tag = "sharing"
)]
#[get("/share/{token}")]
pub async fn resolve_share_link(
    state: web::Data<SharingApiState>,
    path: web::Path<String>,
) -> Result<web::Json<ResolvedShareLinkResponse>, ApiError> {
    let token = path.into_inner();
    let resolved = state.sharing_service.resolve_token(&token)?;
    Ok(web::Json(resolved))
}

pub fn configure_drive(conf: &mut web::ServiceConfig) {
    conf.service(get_file_share_link)
        .service(upsert_file_share_link)
        .service(update_file_share_link)
        .service(delete_file_share_link)
        .service(get_folder_share_link)
        .service(upsert_folder_share_link)
        .service(update_folder_share_link)
        .service(delete_folder_share_link);
}

pub fn configure_public(conf: &mut web::ServiceConfig) {
    conf.service(resolve_share_link);
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_file_share_link,
        upsert_file_share_link,
        update_file_share_link,
        delete_file_share_link,
        get_folder_share_link,
        upsert_folder_share_link,
        update_folder_share_link,
        delete_folder_share_link,
        resolve_share_link,
    ),
    components(schemas(
        ShareLinkResponse,
        UpsertShareLinkRequest,
        UpdateShareLinkRequest,
        ResolvedShareLinkResponse,
        crate::sharing::dto::LinkVisibility,
        crate::sharing::dto::LinkRole,
    )),
    tags((name = "sharing", description = "Link sharing endpoints")),
    modifiers(&SecurityAddon)
)]
pub struct SharingApiDoc;

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
