use crate::persons::{
    dto::{FaceEmbeddingsResponse, ListPersonsResponse, SaveClustersRequest, UsersWithFacesResponse},
    service::PersonsService,
};
use crate::photos::{dto::ListPhotosResponse, service::PhotosService};
use actix_web::{get, post, web, HttpResponse};
use shared::auth::AuthenticatedUser;
use shared::ApiError;
use std::sync::Arc;

pub struct PersonsApiState {
    pub persons_service: Arc<PersonsService>,
    pub photos_service: Arc<PhotosService>,
}

/// List all person clusters for the authenticated user.
#[get("/photos/persons/list")]
pub async fn list_persons(
    state: web::Data<PersonsApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<ListPersonsResponse>, ApiError> {
    let result = state.persons_service.list_persons(&user.user_id)?;
    Ok(web::Json(result))
}

/// Get photos for a specific person cluster.
#[get("/photos/persons/{personId}/photos")]
pub async fn list_person_photos(
    state: web::Data<PersonsApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<web::Json<ListPhotosResponse>, ApiError> {
    let person_id = path.into_inner();
    let photo_ids = state
        .persons_service
        .get_photo_ids_for_person(&person_id, &user.user_id)?;
    let result = state
        .photos_service
        .list_photos_by_ids(&user, &photo_ids)
        .await?;
    Ok(web::Json(result))
}

// ── Internal endpoints (called by the worker, no JWT) ────────────────────────

/// Return all user_ids that have face embeddings (used by worker to trigger cluster-all).
#[get("/internal/users-with-faces")]
pub async fn list_users_with_faces(
    state: web::Data<PersonsApiState>,
) -> Result<web::Json<UsersWithFacesResponse>, ApiError> {
    let result = state.persons_service.list_users_with_face_embeddings()?;
    Ok(web::Json(result))
}

/// Return all face embeddings for a user so the worker can run clustering.
#[get("/internal/users/{userId}/face-embeddings")]
pub async fn get_face_embeddings(
    state: web::Data<PersonsApiState>,
    path: web::Path<String>,
) -> Result<web::Json<FaceEmbeddingsResponse>, ApiError> {
    let user_id = path.into_inner();
    let result = state.persons_service.get_face_embeddings(&user_id)?;
    Ok(web::Json(result))
}

/// Accept clustering results from the worker and persist persons.
#[post("/internal/persons/clusters")]
pub async fn save_clusters(
    state: web::Data<PersonsApiState>,
    body: web::Json<SaveClustersRequest>,
) -> Result<HttpResponse, ApiError> {
    state.persons_service.save_clusters(body.into_inner())?;
    Ok(HttpResponse::NoContent().finish())
}

pub fn configure_persons(cfg: &mut web::ServiceConfig) {
    cfg.service(list_persons)
        .service(list_person_photos)
        .service(list_users_with_faces)
        .service(get_face_embeddings)
        .service(save_clusters);
}
