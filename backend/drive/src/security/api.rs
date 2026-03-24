use actix_web::{delete, get, post, web, HttpResponse};
use std::sync::Arc;
use crate::common::{AdminUser, ApiError};
use crate::security::{
    dto::*,
    service::SecurityService,
};

pub struct SecurityApiState {
    pub service: Arc<SecurityService>,
}

#[get("/security/ransomware/events")]
pub async fn list_ransomware_events(
    state: web::Data<SecurityApiState>,
    _admin: AdminUser,
) -> Result<web::Json<RansomwareEventListResponse>, ApiError> {
    let result = state.service.list_ransomware_events()?;
    Ok(web::Json(result))
}

#[post("/security/ransomware/events/{id}/resolve")]
pub async fn resolve_ransomware_event(
    state: web::Data<SecurityApiState>,
    admin: AdminUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    state.service.resolve_ransomware_event(&path.into_inner(), &admin.user_id)?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/security/siem")]
pub async fn list_siem_configs(
    state: web::Data<SecurityApiState>,
    _admin: AdminUser,
) -> Result<web::Json<SiemConfigListResponse>, ApiError> {
    let result = state.service.list_siem_configs()?;
    Ok(web::Json(result))
}

#[post("/security/siem")]
pub async fn create_siem_config(
    state: web::Data<SecurityApiState>,
    _admin: AdminUser,
    body: web::Json<CreateSiemConfigRequest>,
) -> Result<web::Json<SiemConfigResponse>, ApiError> {
    let result = state.service.create_siem_config(body.into_inner())?;
    Ok(web::Json(result))
}

#[delete("/security/siem/{id}")]
pub async fn delete_siem_config(
    state: web::Data<SecurityApiState>,
    _admin: AdminUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    state.service.delete_siem_config(&path.into_inner())?;
    Ok(HttpResponse::NoContent().finish())
}

#[post("/security/siem/export")]
pub async fn trigger_siem_export(
    state: web::Data<SecurityApiState>,
    _admin: AdminUser,
) -> Result<HttpResponse, ApiError> {
    let count = state.service.export_to_siem()?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "exported": count })))
}

#[post("/security/cmek")]
pub async fn configure_cmek(
    state: web::Data<SecurityApiState>,
    _admin: AdminUser,
    body: web::Json<CmekKeyRequest>,
) -> Result<web::Json<CmekKeyResponse>, ApiError> {
    let result = state.service.configure_cmek(body.into_inner())?;
    Ok(web::Json(result))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_ransomware_events)
        .service(resolve_ransomware_event)
        .service(list_siem_configs)
        .service(create_siem_config)
        .service(delete_siem_config)
        .service(trigger_siem_export)
        .service(configure_cmek);
}
