use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use crate::common::{ApiError, AuthenticatedUser};
use crate::shared_drives::{
    dto::*,
    service::SharedDrivesService,
};

pub struct SharedDrivesApiState {
    pub service: Arc<SharedDrivesService>,
}

#[post("")]
pub async fn create_drive(
    state: web::Data<SharedDrivesApiState>,
    user: AuthenticatedUser,
    body: web::Json<CreateSharedDriveRequest>,
) -> Result<web::Json<SharedDriveResponse>, ApiError> {
    let result = state.service.create(&user, body.into_inner())?;
    Ok(web::Json(result))
}

#[get("")]
pub async fn list_drives(
    state: web::Data<SharedDrivesApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<SharedDriveListResponse>, ApiError> {
    let result = state.service.list_for_user(&user)?;
    Ok(web::Json(result))
}

#[get("/{drive_id}")]
pub async fn get_drive(
    state: web::Data<SharedDrivesApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<web::Json<SharedDriveResponse>, ApiError> {
    let drive_id = path.into_inner();
    let result = state.service.get_by_id(&user, &drive_id)?;
    Ok(web::Json(result))
}

#[patch("/{drive_id}")]
pub async fn update_drive(
    state: web::Data<SharedDrivesApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<UpdateSharedDriveRequest>,
) -> Result<web::Json<SharedDriveResponse>, ApiError> {
    let drive_id = path.into_inner();
    let result = state.service.update(&user, &drive_id, body.into_inner())?;
    Ok(web::Json(result))
}

#[delete("/{drive_id}")]
pub async fn delete_drive(
    state: web::Data<SharedDrivesApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let drive_id = path.into_inner();
    state.service.delete(&user, &drive_id)?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/{drive_id}/members")]
pub async fn list_members(
    state: web::Data<SharedDrivesApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<web::Json<MemberListResponse>, ApiError> {
    let drive_id = path.into_inner();
    let result = state.service.list_members(&user, &drive_id)?;
    Ok(web::Json(result))
}

#[post("/{drive_id}/members")]
pub async fn add_member(
    state: web::Data<SharedDrivesApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<AddMemberRequest>,
) -> Result<web::Json<SharedDriveMemberResponse>, ApiError> {
    let drive_id = path.into_inner();
    let result = state.service.add_member(&user, &drive_id, body.into_inner())?;
    Ok(web::Json(result))
}

#[patch("/{drive_id}/members/{user_id}")]
pub async fn update_member_role(
    state: web::Data<SharedDrivesApiState>,
    user: AuthenticatedUser,
    path: web::Path<(String, String)>,
    body: web::Json<UpdateMemberRoleRequest>,
) -> Result<HttpResponse, ApiError> {
    let (drive_id, target_user_id) = path.into_inner();
    state.service.update_member_role(&user, &drive_id, &target_user_id, body.into_inner())?;
    Ok(HttpResponse::NoContent().finish())
}

#[delete("/{drive_id}/members/{user_id}")]
pub async fn remove_member(
    state: web::Data<SharedDrivesApiState>,
    user: AuthenticatedUser,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (drive_id, target_user_id) = path.into_inner();
    state.service.remove_member(&user, &drive_id, &target_user_id)?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/{drive_id}/analytics")]
pub async fn get_analytics(
    state: web::Data<SharedDrivesApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<web::Json<SharedDriveAnalyticsResponse>, ApiError> {
    let drive_id = path.into_inner();
    let result = state.service.get_analytics(&user, &drive_id)?;
    Ok(web::Json(result))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/shared-drives")
            .service(create_drive)
            .service(list_drives)
            .service(get_drive)
            .service(update_drive)
            .service(delete_drive)
            .service(list_members)
            .service(add_member)
            .service(update_member_role)
            .service(remove_member)
            .service(get_analytics),
    );
}
