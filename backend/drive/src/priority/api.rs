use actix_web::{get, web, HttpResponse};
use crate::common::ApiError;
use crate::common::auth_extractor::AuthenticatedUser;
use super::service::PriorityService;
use std::sync::Arc;

pub struct PriorityApiState {
    pub priority_service: Arc<PriorityService>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(quick_access)
        .service(suggested_collaborators)
        .service(suggested_actions);
}

#[get("/quick-access")]
async fn quick_access(
    state: web::Data<PriorityApiState>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let items = state.priority_service.get_quick_access(&user.user_id, 8)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"items": items})))
}

#[get("/suggested-collaborators")]
async fn suggested_collaborators(
    state: web::Data<PriorityApiState>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let collabs = state
        .priority_service
        .get_suggested_collaborators(&user.user_id)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"collaborators": collabs})))
}

#[get("/files/{file_id}/suggested-actions")]
async fn suggested_actions(
    state: web::Data<PriorityApiState>,
    path: web::Path<String>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let file_id = path.into_inner();
    let actions = state
        .priority_service
        .get_suggested_actions(&user.user_id, &file_id)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"actions": actions})))
}
