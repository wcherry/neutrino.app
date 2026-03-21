use crate::ai::service::SheetsAIService;
use crate::common::{ApiError, AuthenticatedUser};
use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct SheetsAIApiState {
    pub ai_service: Arc<SheetsAIService>,
}

// ── Request / response DTOs ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartFillRequest {
    pub column_values: Vec<String>,
    /// Each element is a [input, output] pair
    pub examples: Vec<[String; 2]>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartFillResponse {
    pub values: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExploreRequest {
    pub question: String,
    pub sheet_data: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PivotRequest {
    pub prompt: String,
    pub sheet_data: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightsRequest {
    pub sheet_data: String,
}

// ── Endpoints ────────────────────────────────────────────────────────────────

#[post("/sheets/{id}/ai/smart-fill")]
pub async fn smart_fill(
    state: web::Data<SheetsAIApiState>,
    _user: AuthenticatedUser,
    _path: web::Path<String>,
    body: web::Json<SmartFillRequest>,
) -> Result<HttpResponse, ApiError> {
    let req = body.into_inner();
    let examples: Vec<(String, String)> = req
        .examples
        .into_iter()
        .map(|pair| (pair[0].clone(), pair[1].clone()))
        .collect();

    let values = state
        .ai_service
        .smart_fill(req.column_values, examples)
        .await
        .map_err(|e| ApiError::new(503, "AI_UNAVAILABLE", e))?;

    Ok(HttpResponse::Ok().json(SmartFillResponse { values }))
}

#[post("/sheets/{id}/ai/explore")]
pub async fn explore(
    state: web::Data<SheetsAIApiState>,
    _user: AuthenticatedUser,
    _path: web::Path<String>,
    body: web::Json<ExploreRequest>,
) -> Result<HttpResponse, ApiError> {
    let req = body.into_inner();

    let result = state
        .ai_service
        .explore(&req.question, &req.sheet_data)
        .await
        .map_err(|e| ApiError::new(503, "AI_UNAVAILABLE", e))?;

    Ok(HttpResponse::Ok().json(result))
}

#[post("/sheets/{id}/ai/pivot")]
pub async fn pivot(
    state: web::Data<SheetsAIApiState>,
    _user: AuthenticatedUser,
    _path: web::Path<String>,
    body: web::Json<PivotRequest>,
) -> Result<HttpResponse, ApiError> {
    let req = body.into_inner();

    let result = state
        .ai_service
        .generate_pivot(&req.prompt, &req.sheet_data)
        .await
        .map_err(|e| ApiError::new(503, "AI_UNAVAILABLE", e))?;

    Ok(HttpResponse::Ok().json(result))
}

#[post("/sheets/{id}/ai/insights")]
pub async fn insights(
    state: web::Data<SheetsAIApiState>,
    _user: AuthenticatedUser,
    _path: web::Path<String>,
    body: web::Json<InsightsRequest>,
) -> Result<HttpResponse, ApiError> {
    let req = body.into_inner();

    let result = state
        .ai_service
        .get_insights(&req.sheet_data)
        .await
        .map_err(|e| ApiError::new(503, "AI_UNAVAILABLE", e))?;

    Ok(HttpResponse::Ok().json(result))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(smart_fill)
        .service(explore)
        .service(pivot)
        .service(insights);
}
