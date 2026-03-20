use crate::notifications::service::NotificationService;
use crate::common::{ApiError, AuthenticatedUser};
use actix_web::{get, post, web, HttpResponse};
use std::sync::Arc;

pub struct NotificationsApiState {
    pub notification_service: Arc<NotificationService>,
}

#[get("/notifications")]
pub async fn list_notifications(
    state: web::Data<NotificationsApiState>,
    user: AuthenticatedUser,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, ApiError> {
    let page = query.get("page").and_then(|p| p.parse().ok());
    let page_size = query.get("pageSize").and_then(|p| p.parse().ok());
    let result = state.notification_service.get_notifications(&user, page, page_size)?;
    Ok(HttpResponse::Ok().json(result))
}

#[post("/notifications/{id}/read")]
pub async fn mark_notification_read(
    state: web::Data<NotificationsApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let notification_id = path.into_inner();
    state.notification_service.mark_read(&user, &notification_id)?;
    Ok(HttpResponse::NoContent().finish())
}

#[post("/notifications/read-all")]
pub async fn mark_all_read(
    state: web::Data<NotificationsApiState>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ApiError> {
    state.notification_service.mark_all_read(&user)?;
    Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_notifications)
        .service(mark_notification_read)
        .service(mark_all_read);
}
