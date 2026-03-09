use crate::features::filesystem::{
    dto::{
        BulkMoveRequest, BulkResult, BulkTrashRequest, CreateFolderRequest, CreateShortcutRequest,
        FolderContentsResponse, FolderResponse, FileResponse, ShortcutResponse,
        TrashContentsResponse, UpdateFileRequest, UpdateFolderRequest,
    },
    service::FilesystemService,
};
use crate::features::shared::{ApiError, AuthenticatedUser};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

pub struct FilesystemApiState {
    pub filesystem_service: Arc<FilesystemService>,
}

// ── Folder endpoints ──────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/fs/folders",
    request_body = CreateFolderRequest,
    responses(
        (status = 201, description = "Folder created", body = FolderResponse),
        (status = 400, description = "Invalid request"),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[post("/folders")]
pub async fn create_folder(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    body: web::Json<CreateFolderRequest>,
) -> Result<HttpResponse, ApiError> {
    let folder = state
        .filesystem_service
        .create_folder(&user.user_id, body.into_inner())?;
    Ok(HttpResponse::Created().json(folder))
}

#[utoipa::path(
    get,
    path = "/api/v1/fs/",
    responses(
        (status = 200, description = "Root folder contents", body = FolderContentsResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[get("/")]
pub async fn get_root_contents(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<FolderContentsResponse>, ApiError> {
    let contents = state
        .filesystem_service
        .get_folder_contents(&user.user_id, None)?;
    Ok(web::Json(contents))
}

#[utoipa::path(
    get,
    path = "/api/v1/fs/folders/{id}",
    params(("id" = String, Path, description = "Folder ID")),
    responses(
        (status = 200, description = "Folder contents", body = FolderContentsResponse),
        (status = 404, description = "Folder not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[get("/folders/{id}")]
pub async fn get_folder_contents(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<web::Json<FolderContentsResponse>, ApiError> {
    let folder_id = path.into_inner();
    let contents = state
        .filesystem_service
        .get_folder_contents(&user.user_id, Some(&folder_id))?;
    Ok(web::Json(contents))
}

#[utoipa::path(
    patch,
    path = "/api/v1/fs/folders/{id}",
    params(("id" = String, Path, description = "Folder ID")),
    request_body = UpdateFolderRequest,
    responses(
        (status = 200, description = "Folder updated", body = FolderResponse),
        (status = 404, description = "Folder not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[patch("/folders/{id}")]
pub async fn update_folder(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<UpdateFolderRequest>,
) -> Result<web::Json<FolderResponse>, ApiError> {
    let folder_id = path.into_inner();
    let folder = state
        .filesystem_service
        .update_folder(&user.user_id, &folder_id, body.into_inner())?;
    Ok(web::Json(folder))
}

#[utoipa::path(
    delete,
    path = "/api/v1/fs/folders/{id}",
    params(("id" = String, Path, description = "Folder ID")),
    responses(
        (status = 204, description = "Folder moved to trash"),
        (status = 404, description = "Folder not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[delete("/folders/{id}")]
pub async fn trash_folder(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let folder_id = path.into_inner();
    state
        .filesystem_service
        .trash_folder(&user.user_id, &folder_id)?;
    Ok(HttpResponse::NoContent().finish())
}

// ── File endpoints ────────────────────────────────────────────────────────────

#[utoipa::path(
    patch,
    path = "/api/v1/fs/files/{id}",
    params(("id" = String, Path, description = "File ID")),
    request_body = UpdateFileRequest,
    responses(
        (status = 200, description = "File updated", body = FileResponse),
        (status = 404, description = "File not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[patch("/files/{id}")]
pub async fn update_file(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<UpdateFileRequest>,
) -> Result<web::Json<FileResponse>, ApiError> {
    let file_id = path.into_inner();
    let file = state
        .filesystem_service
        .update_file(&user.user_id, &file_id, body.into_inner())?;
    Ok(web::Json(file))
}

#[utoipa::path(
    delete,
    path = "/api/v1/fs/files/{id}",
    params(("id" = String, Path, description = "File ID")),
    responses(
        (status = 204, description = "File moved to trash"),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[delete("/files/{id}")]
pub async fn trash_file(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let file_id = path.into_inner();
    state
        .filesystem_service
        .trash_file(&user.user_id, &file_id)?;
    Ok(HttpResponse::NoContent().finish())
}

// ── Shortcut endpoints ────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/fs/shortcuts",
    request_body = CreateShortcutRequest,
    responses(
        (status = 201, description = "Shortcut created", body = ShortcutResponse),
        (status = 400, description = "Invalid request"),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[post("/shortcuts")]
pub async fn create_shortcut(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    body: web::Json<CreateShortcutRequest>,
) -> Result<HttpResponse, ApiError> {
    let shortcut = state
        .filesystem_service
        .create_shortcut(&user.user_id, body.into_inner())?;
    Ok(HttpResponse::Created().json(shortcut))
}

#[utoipa::path(
    delete,
    path = "/api/v1/fs/shortcuts/{id}",
    params(("id" = String, Path, description = "Shortcut ID")),
    responses(
        (status = 204, description = "Shortcut deleted"),
        (status = 404, description = "Shortcut not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[delete("/shortcuts/{id}")]
pub async fn delete_shortcut(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let shortcut_id = path.into_inner();
    state
        .filesystem_service
        .delete_shortcut(&user.user_id, &shortcut_id)?;
    Ok(HttpResponse::NoContent().finish())
}

// ── Bulk endpoints ────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/fs/bulk/move",
    request_body = BulkMoveRequest,
    responses(
        (status = 200, description = "Items moved", body = BulkResult),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[post("/bulk/move")]
pub async fn bulk_move(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    body: web::Json<BulkMoveRequest>,
) -> Result<web::Json<BulkResult>, ApiError> {
    let result = state
        .filesystem_service
        .bulk_move(&user.user_id, body.into_inner())?;
    Ok(web::Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/fs/bulk/trash",
    request_body = BulkTrashRequest,
    responses(
        (status = 200, description = "Items moved to trash", body = BulkResult),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[post("/bulk/trash")]
pub async fn bulk_trash(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    body: web::Json<BulkTrashRequest>,
) -> Result<web::Json<BulkResult>, ApiError> {
    let result = state
        .filesystem_service
        .bulk_trash(&user.user_id, body.into_inner())?;
    Ok(web::Json(result))
}

#[derive(serde::Deserialize)]
pub struct BulkDownloadQuery {
    pub ids: String, // comma-separated file IDs
}

#[utoipa::path(
    get,
    path = "/api/v1/fs/bulk/download",
    params(
        ("ids" = String, Query, description = "Comma-separated file IDs to download as zip"),
    ),
    responses(
        (status = 200, description = "ZIP archive of requested files"),
        (status = 400, description = "No file IDs provided"),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[get("/bulk/download")]
pub async fn bulk_download(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    query: web::Query<BulkDownloadQuery>,
) -> Result<HttpResponse, ApiError> {
    let file_ids: Vec<String> = query
        .ids
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if file_ids.is_empty() {
        return Err(ApiError::bad_request("No file IDs provided"));
    }

    let zip_bytes = state
        .filesystem_service
        .bulk_download(&user.user_id, &file_ids)?;

    Ok(HttpResponse::Ok()
        .content_type("application/zip")
        .insert_header((
            "Content-Disposition",
            "attachment; filename=\"download.zip\"",
        ))
        .body(zip_bytes))
}

// ── Trash endpoints ───────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/fs/trash",
    responses(
        (status = 200, description = "Trashed items", body = TrashContentsResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[get("/trash")]
pub async fn list_trash(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<TrashContentsResponse>, ApiError> {
    let contents = state.filesystem_service.list_trash(&user.user_id)?;
    Ok(web::Json(contents))
}

#[utoipa::path(
    delete,
    path = "/api/v1/fs/trash",
    responses(
        (status = 200, description = "Trash emptied", body = BulkResult),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[delete("/trash")]
pub async fn empty_trash(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<BulkResult>, ApiError> {
    let result = state.filesystem_service.empty_trash(&user.user_id)?;
    Ok(web::Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/fs/trash/files/{id}/restore",
    params(("id" = String, Path, description = "File ID")),
    responses(
        (status = 204, description = "File restored from trash"),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[post("/trash/files/{id}/restore")]
pub async fn restore_file(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let file_id = path.into_inner();
    state
        .filesystem_service
        .restore_file(&user.user_id, &file_id)?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    delete,
    path = "/api/v1/fs/trash/files/{id}",
    params(("id" = String, Path, description = "File ID")),
    responses(
        (status = 204, description = "File permanently deleted"),
        (status = 404, description = "File not found in trash"),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[delete("/trash/files/{id}")]
pub async fn delete_file_permanently(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let file_id = path.into_inner();
    state
        .filesystem_service
        .permanently_delete_file(&user.user_id, &file_id)?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    post,
    path = "/api/v1/fs/trash/folders/{id}/restore",
    params(("id" = String, Path, description = "Folder ID")),
    responses(
        (status = 204, description = "Folder restored from trash"),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[post("/trash/folders/{id}/restore")]
pub async fn restore_folder(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let folder_id = path.into_inner();
    state
        .filesystem_service
        .restore_folder(&user.user_id, &folder_id)?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    delete,
    path = "/api/v1/fs/trash/folders/{id}",
    params(("id" = String, Path, description = "Folder ID")),
    responses(
        (status = 204, description = "Folder permanently deleted"),
        (status = 404, description = "Folder not found in trash"),
    ),
    security(("bearer_auth" = [])),
    tag = "filesystem"
)]
#[delete("/trash/folders/{id}")]
pub async fn delete_folder_permanently(
    state: web::Data<FilesystemApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let folder_id = path.into_inner();
    state
        .filesystem_service
        .permanently_delete_folder(&user.user_id, &folder_id)?;
    Ok(HttpResponse::NoContent().finish())
}

pub fn configure(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/fs")
            .service(create_folder)
            .service(get_root_contents)
            .service(get_folder_contents)
            .service(update_folder)
            .service(trash_folder)
            .service(update_file)
            .service(trash_file)
            .service(create_shortcut)
            .service(delete_shortcut)
            .service(bulk_move)
            .service(bulk_trash)
            .service(bulk_download)
            .service(list_trash)
            .service(empty_trash)
            .service(restore_file)
            .service(delete_file_permanently)
            .service(restore_folder)
            .service(delete_folder_permanently),
    );
}

#[derive(OpenApi)]
#[openapi(
    paths(
        create_folder,
        get_root_contents,
        get_folder_contents,
        update_folder,
        trash_folder,
        update_file,
        trash_file,
        create_shortcut,
        delete_shortcut,
        bulk_move,
        bulk_trash,
        bulk_download,
        list_trash,
        empty_trash,
        restore_file,
        delete_file_permanently,
        restore_folder,
        delete_folder_permanently,
    ),
    components(schemas(
        CreateFolderRequest,
        UpdateFolderRequest,
        FolderResponse,
        FolderContentsResponse,
        UpdateFileRequest,
        FileResponse,
        CreateShortcutRequest,
        ShortcutResponse,
        BulkMoveRequest,
        BulkTrashRequest,
        BulkResult,
        TrashContentsResponse,
        crate::features::filesystem::dto::TrashFileItem,
        crate::features::filesystem::dto::TrashFolderItem,
    )),
    tags((name = "filesystem", description = "File system organization endpoints")),
    modifiers(&SecurityAddon)
)]
pub struct FilesystemApiDoc;

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
