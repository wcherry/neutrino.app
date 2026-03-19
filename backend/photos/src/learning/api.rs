use crate::learning::{dto::{ReprocessingResponse, ThresholdsResponse}, service::LearningService};
use actix_web::{get, post, web};
use shared::auth::AuthenticatedUser;
use shared::ApiError;
use std::sync::Arc;

pub struct LearningApiState {
    pub learning_service: Arc<LearningService>,
}

pub fn configure_learning(cfg: &mut web::ServiceConfig) {
    cfg.service(get_thresholds).service(trigger_reprocess);
}

/// GET /api/v1/photos/learning/thresholds
#[get("/photos/learning/thresholds")]
async fn get_thresholds(
    state: web::Data<LearningApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<ThresholdsResponse>, ApiError> {
    let resp = state.learning_service.get_thresholds(&user.user_id)?;
    Ok(web::Json(resp))
}

/// POST /api/v1/photos/learning/reprocess
#[post("/photos/learning/reprocess")]
async fn trigger_reprocess(
    state: web::Data<LearningApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<ReprocessingResponse>, ApiError> {
    let resp = state.learning_service.process_pending_for_user(&user.user_id)?;
    Ok(web::Json(resp))
}
