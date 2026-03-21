use actix_web::{get, post, web, HttpResponse};
use crate::common::ApiError;
use crate::common::auth_extractor::AuthenticatedUser;
use super::service::DriveAIService;
use crate::search::service::SearchService;
use std::sync::Arc;
use serde::Deserialize;

pub struct DriveAIApiState {
    pub ai_service: Arc<DriveAIService>,
    pub search_service: Arc<SearchService>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_file_summary)
        .service(catch_me_up)
        .service(ask_drive);
}

#[get("/files/{file_id}/summary")]
async fn get_file_summary(
    state: web::Data<DriveAIApiState>,
    path: web::Path<String>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let file_id = path.into_inner();
    let summary = state
        .ai_service
        .get_file_summary(&file_id, &user.user_id)
        .await?;
    Ok(HttpResponse::Ok().json(summary))
}

#[get("/catch-me-up")]
async fn catch_me_up(
    state: web::Data<DriveAIApiState>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let result = state.ai_service.catch_me_up(&user.user_id).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AskRequest {
    question: String,
}

#[post("/ask")]
async fn ask_drive(
    state: web::Data<DriveAIApiState>,
    user: AuthenticatedUser,
    body: web::Json<AskRequest>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .ai_service
        .answer_question(&user.user_id, &body.question, &state.search_service)
        .await?;
    Ok(HttpResponse::Ok().json(result))
}
