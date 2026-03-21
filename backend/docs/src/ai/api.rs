use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;

use super::service::{DocsAIService, GrammarIssue};
use crate::common::{ApiError, AuthenticatedUser};

pub struct DocsAIState {
    pub ai_service: Arc<DocsAIService>,
}

// ── Request bodies ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartComposeRequest {
    pub context: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrammarCheckRequest {
    pub text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateRequest {
    pub content: String,
    pub target_lang: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HelpMeWriteRequest {
    pub description: String,
}

// ── Response bodies ───────────────────────────────────────────────────────────

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionResponse {
    pub completion: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrammarResponse {
    pub issues: Vec<GrammarIssue>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateResponse {
    pub translated: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummarizeResponse {
    pub summary: String,
}

// ── Endpoints ─────────────────────────────────────────────────────────────────

/// POST /api/v1/docs/{id}/ai/complete
#[post("/docs/{id}/ai/complete")]
pub async fn smart_compose(
    state: web::Data<DocsAIState>,
    _user: AuthenticatedUser,
    _path: web::Path<String>,
    body: web::Json<SmartComposeRequest>,
) -> Result<web::Json<CompletionResponse>, ApiError> {
    let completion = state.ai_service.smart_compose(&body.context).await?;
    Ok(web::Json(CompletionResponse { completion }))
}

/// POST /api/v1/docs/{id}/ai/grammar
#[post("/docs/{id}/ai/grammar")]
pub async fn grammar_check(
    state: web::Data<DocsAIState>,
    _user: AuthenticatedUser,
    _path: web::Path<String>,
    body: web::Json<GrammarCheckRequest>,
) -> Result<web::Json<GrammarResponse>, ApiError> {
    let issues = state.ai_service.grammar_check(&body.text).await?;
    Ok(web::Json(GrammarResponse { issues }))
}

/// POST /api/v1/docs/{id}/ai/translate
#[post("/docs/{id}/ai/translate")]
pub async fn translate(
    state: web::Data<DocsAIState>,
    _user: AuthenticatedUser,
    _path: web::Path<String>,
    body: web::Json<TranslateRequest>,
) -> Result<web::Json<TranslateResponse>, ApiError> {
    let translated = state
        .ai_service
        .translate(&body.content, &body.target_lang)
        .await?;
    Ok(web::Json(TranslateResponse { translated }))
}

/// POST /api/v1/docs/ai/help-me-write
#[post("/docs/ai/help-me-write")]
pub async fn help_me_write(
    state: web::Data<DocsAIState>,
    _user: AuthenticatedUser,
    body: web::Json<HelpMeWriteRequest>,
) -> Result<web::Json<CompletionResponse>, ApiError> {
    let completion = state.ai_service.help_me_write(&body.description).await?;
    Ok(web::Json(CompletionResponse { completion }))
}

/// GET /api/v1/docs/{id}/ai/summarize
#[get("/docs/{id}/ai/summarize")]
pub async fn summarize(
    _state: web::Data<DocsAIState>,
    _user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    // We can't get content from here without the doc content — return 400 if no text query param
    // For the summarize endpoint, require a ?text= query param OR use the doc ID to fetch content
    // Following the pattern of the docs service, we accept a ?text= param for simplicity
    let _ = path.into_inner();
    Ok(HttpResponse::BadRequest().json(serde_json::json!({
        "error": {
            "code": "USE_POST",
            "message": "Use POST /docs/{id}/ai/summarize with body {text: string}"
        }
    })))
}

/// POST /api/v1/docs/{id}/ai/summarize
#[post("/docs/{id}/ai/summarize")]
pub async fn summarize_post(
    state: web::Data<DocsAIState>,
    _user: AuthenticatedUser,
    _path: web::Path<String>,
    body: web::Json<serde_json::Value>,
) -> Result<web::Json<SummarizeResponse>, ApiError> {
    let content = body
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let summary = state.ai_service.summarize(content).await?;
    Ok(web::Json(SummarizeResponse { summary }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(help_me_write)
        .service(smart_compose)
        .service(grammar_check)
        .service(translate)
        .service(summarize)
        .service(summarize_post);
}
