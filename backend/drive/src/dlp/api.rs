use actix_web::{delete, get, post, web, HttpResponse};
use std::sync::Arc;
use crate::common::{AdminUser, ApiError};
use crate::dlp::{
    dto::*,
    service::DlpService,
};

pub struct DlpApiState {
    pub service: Arc<DlpService>,
}

#[get("/dlp/rules")]
pub async fn list_rules(
    state: web::Data<DlpApiState>,
    _admin: AdminUser,
) -> Result<web::Json<DlpRuleListResponse>, ApiError> {
    let result = state.service.list_rules()?;
    Ok(web::Json(result))
}

#[post("/dlp/rules")]
pub async fn create_rule(
    state: web::Data<DlpApiState>,
    admin: AdminUser,
    body: web::Json<CreateDlpRuleRequest>,
) -> Result<web::Json<DlpRuleResponse>, ApiError> {
    let result = state.service.create_rule(&admin.user_id, body.into_inner())?;
    Ok(web::Json(result))
}

#[get("/dlp/rules/{id}")]
pub async fn get_rule(
    state: web::Data<DlpApiState>,
    _admin: AdminUser,
    path: web::Path<String>,
) -> Result<web::Json<DlpRuleResponse>, ApiError> {
    let id = path.into_inner();
    let result = state.service.get_rule(&id)?;
    Ok(web::Json(result))
}

#[delete("/dlp/rules/{id}")]
pub async fn delete_rule(
    state: web::Data<DlpApiState>,
    _admin: AdminUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    state.service.delete_rule(&id)?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/dlp/violations")]
pub async fn list_violations(
    state: web::Data<DlpApiState>,
    _admin: AdminUser,
    query: web::Query<DlpViolationQuery>,
) -> Result<web::Json<DlpViolationListResponse>, ApiError> {
    let result = state.service.list_violations(&query)?;
    Ok(web::Json(result))
}

#[post("/dlp/violations/{id}/dismiss")]
pub async fn dismiss_violation(
    state: web::Data<DlpApiState>,
    admin: AdminUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    state.service.dismiss_violation(&id, &admin.user_id)?;
    Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_rules)
        .service(create_rule)
        .service(get_rule)
        .service(delete_rule)
        .service(list_violations)
        .service(dismiss_violation);
}
