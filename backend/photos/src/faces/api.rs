use crate::faces::{
    dto::{FaceResponse, ListFacesResponse, SaveFaceRequest},
    service::FacesService,
};
use actix_web::{get, post, web, HttpResponse};
use shared::auth::AuthenticatedUser;
use shared::ApiError;
use std::sync::Arc;

pub struct FacesApiState {
    pub faces_service: Arc<FacesService>,
}

/// List faces detected in a photo.
#[get("/photos/{photoId}/faces")]
pub async fn list_faces(
    state: web::Data<FacesApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<web::Json<ListFacesResponse>, ApiError> {
    let photo_id = path.into_inner();
    let result = state.faces_service.list_faces(&photo_id, &user.user_id)?;
    Ok(web::Json(result))
}

/// Worker endpoint — saves a single detected face for a photo.
/// No user auth: called by the background worker after face detection.
#[post("/photos/{photoId}/faces")]
pub async fn save_face(
    state: web::Data<FacesApiState>,
    path: web::Path<String>,
    body: web::Json<SaveFaceRequest>,
) -> Result<HttpResponse, ApiError> {
    let photo_id = path.into_inner();
    let face: FaceResponse = state.faces_service.save_face(&photo_id, body.into_inner())?;
    Ok(HttpResponse::Created().json(face))
}

pub fn configure_faces(cfg: &mut web::ServiceConfig) {
    cfg.service(list_faces).service(save_face);
}
