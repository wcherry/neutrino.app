use actix_web::{get, put, web, HttpResponse};
use crate::common::ApiError;
use crate::common::auth_extractor::AuthenticatedUser;
use super::dto::*;
use super::service::SearchService;
use std::sync::Arc;

pub struct SearchApiState {
    pub search_service: Arc<SearchService>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(search_files).service(index_file_content);
}

#[get("/search")]
async fn search_files(
    state: web::Data<SearchApiState>,
    query: web::Query<SearchQuery>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    let results = state.search_service.search(&user.user_id, &query)?;
    Ok(HttpResponse::Ok().json(results))
}

#[put("/jobs/files/{file_id}/content-index")]
async fn index_file_content(
    state: web::Data<SearchApiState>,
    path: web::Path<String>,
    user: AuthenticatedUser,
    body: web::Json<ContentIndexRequest>,
) -> Result<HttpResponse, ApiError> {
    let file_id = path.into_inner();
    state
        .search_service
        .upsert_content_index(&file_id, &user.user_id, &body.text_content)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"indexed": true})))
}
