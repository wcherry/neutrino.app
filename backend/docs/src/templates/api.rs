use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;

use super::{
    dto::{CreateTemplateRequest, ListTemplatesResponse, TemplateResponse, UpdateTemplateRequest},
    service::TemplatesService,
};
use crate::common::{ApiError, AuthenticatedUser};

pub struct TemplatesApiState {
    pub templates_service: Arc<TemplatesService>,
}

// ── Endpoints ─────────────────────────────────────────────────────────────────

/// GET /api/v1/docs/templates
#[get("/docs/templates")]
pub async fn list_templates(
    state: web::Data<TemplatesApiState>,
    _user: AuthenticatedUser,
) -> Result<web::Json<ListTemplatesResponse>, ApiError> {
    let result = state.templates_service.list_templates()?;
    Ok(web::Json(result))
}

/// POST /api/v1/docs/templates
#[post("/docs/templates")]
pub async fn create_template(
    state: web::Data<TemplatesApiState>,
    _user: AuthenticatedUser,
    body: web::Json<CreateTemplateRequest>,
) -> Result<HttpResponse, ApiError> {
    let template = state.templates_service.create_template(body.into_inner())?;
    Ok(HttpResponse::Created().json(template))
}

/// GET /api/v1/docs/templates/{id}
#[get("/docs/templates/{id}")]
pub async fn get_template(
    state: web::Data<TemplatesApiState>,
    _user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<web::Json<TemplateResponse>, ApiError> {
    let id = path.into_inner();
    let template = state.templates_service.get_template(&id)?;
    Ok(web::Json(template))
}

/// PATCH /api/v1/docs/templates/{id}
#[patch("/docs/templates/{id}")]
pub async fn update_template(
    state: web::Data<TemplatesApiState>,
    _user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<UpdateTemplateRequest>,
) -> Result<web::Json<TemplateResponse>, ApiError> {
    let id = path.into_inner();
    let template = state.templates_service.update_template(&id, body.into_inner())?;
    Ok(web::Json(template))
}

/// DELETE /api/v1/docs/templates/{id}
#[delete("/docs/templates/{id}")]
pub async fn delete_template(
    state: web::Data<TemplatesApiState>,
    _user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    state.templates_service.delete_template(&id)?;
    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/v1/docs/templates/{id}/use
#[post("/docs/templates/{id}/use")]
pub async fn use_template(
    state: web::Data<TemplatesApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let title = body
        .get("title")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let response = state.templates_service.use_template(&id, &user, title).await?;
    Ok(HttpResponse::Created().json(response))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_templates)
        .service(create_template)
        .service(get_template)
        .service(update_template)
        .service(delete_template)
        .service(use_template);
}
