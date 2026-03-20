use crate::suggestions::{
    dto::CreateSuggestionRequest,
    service::SuggestionsService,
};
use crate::common::{ApiError, AuthenticatedUser};
use actix_web::{get, post, web, HttpResponse};
use std::sync::Arc;

pub struct SuggestionsApiState {
    pub suggestions_service: Arc<SuggestionsService>,
}

#[get("/files/{id}/suggestions")]
pub async fn list_suggestions(
    state: web::Data<SuggestionsApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, ApiError> {
    let file_id = path.into_inner();
    let status = query.get("status").map(|s| s.as_str());
    let result = state.suggestions_service.list_suggestions(&user, &file_id, status)?;
    Ok(HttpResponse::Ok().json(result))
}

#[post("/files/{id}/suggestions")]
pub async fn create_suggestion(
    state: web::Data<SuggestionsApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<CreateSuggestionRequest>,
) -> Result<HttpResponse, ApiError> {
    let file_id = path.into_inner();
    let result = state.suggestions_service.create_suggestion(&user, &file_id, body.into_inner())?;
    Ok(HttpResponse::Created().json(result))
}

#[post("/files/{id}/suggestions/{sid}/accept")]
pub async fn accept_suggestion(
    state: web::Data<SuggestionsApiState>,
    user: AuthenticatedUser,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (file_id, suggestion_id) = path.into_inner();
    let result = state.suggestions_service.accept_suggestion(&user, &file_id, &suggestion_id).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[post("/files/{id}/suggestions/{sid}/reject")]
pub async fn reject_suggestion(
    state: web::Data<SuggestionsApiState>,
    user: AuthenticatedUser,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (file_id, suggestion_id) = path.into_inner();
    let result = state.suggestions_service.reject_suggestion(&user, &file_id, &suggestion_id).await?;
    Ok(HttpResponse::Ok().json(result))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_suggestions)
        .service(create_suggestion)
        .service(accept_suggestion)
        .service(reject_suggestion);
}
