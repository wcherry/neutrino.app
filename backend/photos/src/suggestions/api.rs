use crate::suggestions::{dto::ListSuggestionsResponse, service::SuggestionsService};
use actix_web::{get, post, web, HttpResponse};
use shared::auth::AuthenticatedUser;
use shared::ApiError;
use std::sync::Arc;

pub struct SuggestionsApiState {
    pub suggestions_service: Arc<SuggestionsService>,
}

/// List all pending face suggestions for the authenticated user.
#[get("/photos/suggestions")]
pub async fn list_suggestions(
    state: web::Data<SuggestionsApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<ListSuggestionsResponse>, ApiError> {
    let result = state.suggestions_service.list_suggestions(&user)?;
    Ok(web::Json(result))
}

/// Accept a suggestion: assign the face to the suggested person.
#[post("/photos/suggestions/{id}/accept")]
pub async fn accept_suggestion(
    state: web::Data<SuggestionsApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    state.suggestions_service.accept_suggestion(&user, &id)?;
    Ok(HttpResponse::NoContent().finish())
}

/// Reject a suggestion: prevents this face from being re-suggested for this person.
#[post("/photos/suggestions/{id}/reject")]
pub async fn reject_suggestion(
    state: web::Data<SuggestionsApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    state.suggestions_service.reject_suggestion(&user, &id)?;
    Ok(HttpResponse::NoContent().finish())
}

pub fn configure_suggestions(cfg: &mut web::ServiceConfig) {
    cfg.service(list_suggestions)
        .service(accept_suggestion)
        .service(reject_suggestion);
}
