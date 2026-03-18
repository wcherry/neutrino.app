use crate::persons::{
    dto::{
        ClusterEntry, FaceEmbeddingEntry, FaceEmbeddingsResponse, ListPersonsResponse,
        PersonFaceThumbnail, PersonResponse, SaveClustersRequest, UsersWithFacesResponse,
    },
    repository::PersonsRepository,
};
use shared::ApiError;
use std::sync::Arc;
use uuid::Uuid;

pub struct PersonsService {
    repo: Arc<PersonsRepository>,
}

impl PersonsService {
    pub fn new(repo: Arc<PersonsRepository>) -> Self {
        PersonsService { repo }
    }

    pub fn list_persons(&self, user_id: &str) -> Result<ListPersonsResponse, ApiError> {
        let records = self.repo.list_persons_for_user(user_id)?;
        let total = records.len();

        // Load all faces for these persons in one query, then group by person_id.
        let person_ids: Vec<String> = records.iter().map(|p| p.id.clone()).collect();
        let all_faces = if person_ids.is_empty() {
            vec![]
        } else {
            self.repo.list_faces_for_persons(&person_ids)?
        };

        use std::collections::HashMap;
        let mut faces_by_person: HashMap<String, Vec<PersonFaceThumbnail>> = HashMap::new();
        for face in all_faces {
            if let Some(pid) = &face.person_id {
                faces_by_person.entry(pid.clone()).or_default().push(PersonFaceThumbnail {
                    id: face.id,
                    thumbnail: face.thumbnail,
                    thumbnail_mime_type: face.thumbnail_mime_type,
                });
            }
        }

        let persons = records
            .into_iter()
            .map(|p| {
                let faces = faces_by_person.remove(&p.id).unwrap_or_default();
                PersonResponse {
                    id: p.id,
                    cover_face_id: p.cover_face_id,
                    cover_thumbnail: p.cover_thumbnail,
                    cover_thumbnail_mime_type: p.cover_thumbnail_mime_type,
                    face_count: p.face_count,
                    faces,
                    created_at: p.created_at.and_utc().to_rfc3339(),
                    updated_at: p.updated_at.and_utc().to_rfc3339(),
                }
            })
            .collect();
        Ok(ListPersonsResponse { persons, total })
    }

    /// Returns all user_ids that have at least one face embedding (called by the worker to trigger cluster-all).
    pub fn list_users_with_face_embeddings(&self) -> Result<UsersWithFacesResponse, ApiError> {
        let user_ids = self.repo.list_users_with_face_embeddings()?;
        Ok(UsersWithFacesResponse { user_ids })
    }

    /// Returns all face embeddings for a user (called by the worker before clustering).
    pub fn get_face_embeddings(&self, user_id: &str) -> Result<FaceEmbeddingsResponse, ApiError> {
        let face_records = self.repo.list_face_embeddings_for_user(user_id)?;
        let faces = face_records
            .into_iter()
            .filter_map(|f| {
                let embedding_json = f.embedding?;
                let embedding: Vec<f32> = serde_json::from_str(&embedding_json).ok()?;
                Some(FaceEmbeddingEntry {
                    face_id: f.id,
                    photo_id: f.photo_id,
                    embedding,
                    thumbnail: f.thumbnail,
                    thumbnail_mime_type: f.thumbnail_mime_type,
                })
            })
            .collect();
        Ok(FaceEmbeddingsResponse { faces })
    }

    /// Save clustering results from the worker.
    pub fn save_clusters(&self, req: SaveClustersRequest) -> Result<(), ApiError> {
        let now = chrono::Utc::now().naive_utc();
        let clusters: Vec<(String, Vec<String>, Option<String>, Option<String>, Option<String>)> =
            req.clusters
                .into_iter()
                .map(|c: ClusterEntry| {
                    let person_id = Uuid::new_v4().to_string();
                    (
                        person_id,
                        c.face_ids,
                        Some(c.cover_face_id),
                        c.cover_thumbnail,
                        c.cover_thumbnail_mime_type,
                    )
                })
                .collect();
        self.repo.apply_clusters(&req.user_id, &clusters, now)
    }

    /// Returns distinct photo IDs for photos that contain this person's faces.
    pub fn get_photo_ids_for_person(
        &self,
        person_id: &str,
        user_id: &str,
    ) -> Result<Vec<String>, ApiError> {
        let person = self.repo.get_person(person_id)?;
        if person.user_id != user_id {
            return Err(ApiError::new(403, "FORBIDDEN", "Access denied"));
        }
        let face_records = self.repo.list_faces_for_person(person_id)?;
        let mut photo_ids: Vec<String> =
            face_records.into_iter().map(|f| f.photo_id).collect();
        photo_ids.sort();
        photo_ids.dedup();
        Ok(photo_ids)
    }
}
