use crate::ai::service::SlidesAIService;
use crate::common::{ApiError, AuthenticatedUser};
use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;

pub struct SlidesAIApiState {
    pub ai_service: Arc<SlidesAIService>,
}

// ── Request DTOs ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartComposeRequest {
    pub slide_text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSearchRequest {
    pub query: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignRequest {
    pub slide_content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoFormatRequest {
    pub slide_json: String,
}

// ── Endpoints ────────────────────────────────────────────────────────────────

#[post("/slides/{id}/ai/complete")]
pub async fn smart_compose(
    state: web::Data<SlidesAIApiState>,
    _user: AuthenticatedUser,
    _path: web::Path<String>,
    body: web::Json<SmartComposeRequest>,
) -> Result<HttpResponse, ApiError> {
    let req = body.into_inner();

    let completed = state
        .ai_service
        .smart_compose(&req.slide_text)
        .await
        .map_err(|e| ApiError::new(503, "AI_UNAVAILABLE", e))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "text": completed })))
}

#[post("/slides/{id}/ai/image-search")]
pub async fn image_search(
    state: web::Data<SlidesAIApiState>,
    user: AuthenticatedUser,
    _path: web::Path<String>,
    body: web::Json<ImageSearchRequest>,
) -> Result<HttpResponse, ApiError> {
    let query_req = body.into_inner();

    let results = state
        .ai_service
        .search_images(&query_req.query, &user.token)
        .await
        .map_err(|e| ApiError::new(503, "AI_UNAVAILABLE", e))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "images": results })))
}

#[post("/slides/{id}/ai/design")]
pub async fn help_design(
    state: web::Data<SlidesAIApiState>,
    _user: AuthenticatedUser,
    _path: web::Path<String>,
    body: web::Json<DesignRequest>,
) -> Result<HttpResponse, ApiError> {
    let req = body.into_inner();

    let result = state
        .ai_service
        .help_design(&req.slide_content)
        .await
        .map_err(|e| ApiError::new(503, "AI_UNAVAILABLE", e))?;

    Ok(HttpResponse::Ok().json(result))
}

#[post("/slides/{id}/ai/autoformat")]
pub async fn auto_format(
    state: web::Data<SlidesAIApiState>,
    _user: AuthenticatedUser,
    _path: web::Path<String>,
    body: web::Json<AutoFormatRequest>,
) -> Result<HttpResponse, ApiError> {
    let req = body.into_inner();

    let result = state
        .ai_service
        .auto_format(&req.slide_json)
        .await
        .map_err(|e| ApiError::new(503, "AI_UNAVAILABLE", e))?;

    Ok(HttpResponse::Ok().json(result))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(smart_compose)
        .service(image_search)
        .service(help_design)
        .service(auto_format);
}
