use crate::albums::api::AlbumsApiState;
use crate::persons::{
    dto::{
        FaceEmbeddingsResponse, ListPersonsResponse, MergePersonsRequest,
        PersonRelationshipsResponse, PersonResponse, PersonTimelineResponse, ReassignFaceRequest,
        RenamePersonRequest, SaveClustersRequest, UsersWithFacesResponse,
    },
    service::PersonsService,
};
use crate::photos::{dto::ListPhotosResponse, service::PhotosService};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use shared::auth::AuthenticatedUser;
use shared::ApiError;
use std::sync::Arc;

pub struct PersonsApiState {
    pub persons_service: Arc<PersonsService>,
    pub photos_service: Arc<PhotosService>,
    pub albums_service: Arc<crate::albums::service::AlbumsService>,
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

/// Rename a person cluster.
#[patch("/photos/persons/{personId}")]
pub async fn rename_person(
    state: web::Data<PersonsApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<RenamePersonRequest>,
) -> Result<web::Json<PersonResponse>, ApiError> {
    let person_id = path.into_inner();
    let result = state
        .persons_service
        .rename_person(&person_id, &user.user_id, body.into_inner())?;
    Ok(web::Json(result))
}

/// Merge another person cluster into this one (source is absorbed and deleted).
#[post("/photos/persons/{personId}/merge")]
pub async fn merge_persons(
    state: web::Data<PersonsApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<MergePersonsRequest>,
) -> Result<web::Json<PersonResponse>, ApiError> {
    let target_id = path.into_inner();
    let result = state
        .persons_service
        .merge_persons(&target_id, &user.user_id, body.into_inner())?;
    Ok(web::Json(result))
}

/// Move a face from this person to a different person.
#[patch("/photos/persons/{personId}/faces/{faceId}")]
pub async fn reassign_face(
    state: web::Data<PersonsApiState>,
    user: AuthenticatedUser,
    path: web::Path<(String, String)>,
    body: web::Json<ReassignFaceRequest>,
) -> Result<HttpResponse, ApiError> {
    let (person_id, face_id) = path.into_inner();
    state
        .persons_service
        .reassign_face(&person_id, &face_id, &user.user_id, body.into_inner())?;
    Ok(HttpResponse::NoContent().finish())
}

/// Remove a face from this person (unassigns it; person deleted if now empty).
#[delete("/photos/persons/{personId}/faces/{faceId}")]
pub async fn remove_face(
    state: web::Data<PersonsApiState>,
    user: AuthenticatedUser,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (person_id, face_id) = path.into_inner();
    state
        .persons_service
        .remove_face_from_person(&person_id, &face_id, &user.user_id)?;
    Ok(HttpResponse::NoContent().finish())
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

/// Get photos of a person in chronological timeline groups.
#[get("/photos/persons/{personId}/timeline")]
pub async fn get_person_timeline(
    state: web::Data<PersonsApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<web::Json<PersonTimelineResponse>, ApiError> {
    let person_id = path.into_inner();
    // Fetch photo IDs for this person.
    let photo_ids = state
        .persons_service
        .get_photo_ids_for_person(&person_id, &user.user_id)?;
    // Resolve photos (including Drive file info) via PhotosService.
    let photos_resp = state
        .photos_service
        .list_photos_by_ids(&user, &photo_ids)
        .await?;
    let result = state
        .persons_service
        .build_timeline(&person_id, &user.user_id, photos_resp.photos)?;
    Ok(web::Json(result))
}

/// Get relationship insights: persons who frequently co-appear in photos.
#[get("/photos/persons/relationships")]
pub async fn get_person_relationships(
    state: web::Data<PersonsApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<PersonRelationshipsResponse>, ApiError> {
    let result = state.persons_service.get_relationships(&user.user_id)?;
    Ok(web::Json(result))
}

/// Create or refresh a smart album for a named person.
#[post("/photos/persons/{personId}/smart-album")]
pub async fn create_person_smart_album(
    state: web::Data<PersonsApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let person_id = path.into_inner();
    let person = state
        .persons_service
        .get_person_for_user(&person_id, &user.user_id)?;
    let person_name = person
        .name
        .as_deref()
        .unwrap_or("Unknown person");
    let photo_ids = state
        .persons_service
        .get_photo_ids_for_person(&person_id, &user.user_id)?;
    let album = state
        .albums_service
        .upsert_person_smart_album(&user.user_id, &person_id, person_name, &photo_ids)?;
    Ok(HttpResponse::Ok().json(album))
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
        .service(get_person_relationships)
        .service(rename_person)
        .service(merge_persons)
        .service(reassign_face)
        .service(remove_face)
        .service(list_person_photos)
        .service(get_person_timeline)
        .service(create_person_smart_album)
        .service(list_users_with_faces)
        .service(get_face_embeddings)
        .service(save_clusters);
}
