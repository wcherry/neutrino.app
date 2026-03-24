use actix_web::{delete, get, post, put, web, HttpResponse};
use std::sync::Arc;
use crate::common::{AdminUser, ApiError};
use crate::compliance::{
    dto::*,
    service::ComplianceService,
};

pub struct ComplianceApiState {
    pub service: Arc<ComplianceService>,
}

// Legal Holds
#[get("/compliance/holds")]
pub async fn list_holds(
    state: web::Data<ComplianceApiState>,
    _admin: AdminUser,
) -> Result<web::Json<LegalHoldListResponse>, ApiError> {
    let result = state.service.list_holds()?;
    Ok(web::Json(result))
}

#[post("/compliance/holds")]
pub async fn create_hold(
    state: web::Data<ComplianceApiState>,
    admin: AdminUser,
    body: web::Json<CreateLegalHoldRequest>,
) -> Result<web::Json<LegalHoldResponse>, ApiError> {
    let result = state.service.create_hold(&admin.user_id, body.into_inner())?;
    Ok(web::Json(result))
}

#[get("/compliance/holds/{id}")]
pub async fn get_hold(
    state: web::Data<ComplianceApiState>,
    _admin: AdminUser,
    path: web::Path<String>,
) -> Result<web::Json<LegalHoldResponse>, ApiError> {
    let result = state.service.get_hold(&path.into_inner())?;
    Ok(web::Json(result))
}

#[put("/compliance/holds/{id}")]
pub async fn update_hold(
    state: web::Data<ComplianceApiState>,
    _admin: AdminUser,
    path: web::Path<String>,
    body: web::Json<UpdateLegalHoldRequest>,
) -> Result<web::Json<LegalHoldResponse>, ApiError> {
    let result = state.service.update_hold(&path.into_inner(), body.into_inner())?;
    Ok(web::Json(result))
}

#[delete("/compliance/holds/{id}")]
pub async fn delete_hold(
    state: web::Data<ComplianceApiState>,
    _admin: AdminUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    state.service.delete_hold(&path.into_inner())?;
    Ok(HttpResponse::NoContent().finish())
}

#[post("/compliance/holds/{id}/files/{file_id}")]
pub async fn apply_hold_to_file(
    state: web::Data<ComplianceApiState>,
    _admin: AdminUser,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (hold_id, file_id) = path.into_inner();
    state.service.apply_hold_to_file(&hold_id, &file_id)?;
    Ok(HttpResponse::NoContent().finish())
}

#[delete("/compliance/holds/{id}/files/{file_id}")]
pub async fn remove_hold_from_file(
    state: web::Data<ComplianceApiState>,
    _admin: AdminUser,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (hold_id, file_id) = path.into_inner();
    state.service.remove_hold_from_file(&hold_id, &file_id)?;
    Ok(HttpResponse::NoContent().finish())
}

// Retention Policies
#[get("/compliance/retention")]
pub async fn list_policies(
    state: web::Data<ComplianceApiState>,
    _admin: AdminUser,
) -> Result<web::Json<RetentionPolicyListResponse>, ApiError> {
    let result = state.service.list_policies()?;
    Ok(web::Json(result))
}

#[post("/compliance/retention")]
pub async fn create_policy(
    state: web::Data<ComplianceApiState>,
    _admin: AdminUser,
    body: web::Json<CreateRetentionPolicyRequest>,
) -> Result<web::Json<RetentionPolicyResponse>, ApiError> {
    let result = state.service.create_policy(body.into_inner())?;
    Ok(web::Json(result))
}

#[get("/compliance/retention/{id}")]
pub async fn get_policy(
    state: web::Data<ComplianceApiState>,
    _admin: AdminUser,
    path: web::Path<String>,
) -> Result<web::Json<RetentionPolicyResponse>, ApiError> {
    let result = state.service.get_policy(&path.into_inner())?;
    Ok(web::Json(result))
}

#[delete("/compliance/retention/{id}")]
pub async fn delete_policy(
    state: web::Data<ComplianceApiState>,
    _admin: AdminUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    state.service.delete_policy(&path.into_inner())?;
    Ok(HttpResponse::NoContent().finish())
}

// eDiscovery
#[post("/compliance/ediscovery/search")]
pub async fn ediscovery_search(
    state: web::Data<ComplianceApiState>,
    _admin: AdminUser,
    body: web::Json<EDiscoverySearchRequest>,
) -> Result<web::Json<EDiscoverySearchResponse>, ApiError> {
    let result = state.service.ediscovery_search(body.into_inner())?;
    Ok(web::Json(result))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_holds)
        .service(create_hold)
        .service(get_hold)
        .service(update_hold)
        .service(delete_hold)
        .service(apply_hold_to_file)
        .service(remove_hold_from_file)
        .service(list_policies)
        .service(create_policy)
        .service(get_policy)
        .service(delete_policy)
        .service(ediscovery_search);
}
