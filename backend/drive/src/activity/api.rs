use crate::activity::service::ActivityService;
use crate::common::{ApiError, AuthenticatedUser};
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;

pub struct ActivityApiState {
    pub activity_service: Arc<ActivityService>,
}

#[get("/files/{id}/activity")]
pub async fn list_file_activity(
    state: web::Data<ActivityApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, ApiError> {
    let file_id = path.into_inner();
    let page = query.get("page").and_then(|p| p.parse().ok());
    let page_size = query.get("pageSize").and_then(|p| p.parse().ok());
    let result = state.activity_service.list_file_activity(&user, &file_id, page, page_size)?;
    Ok(HttpResponse::Ok().json(result))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_file_activity);
}
