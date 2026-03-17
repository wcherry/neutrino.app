use crate::photos::{
    dto::{ListPhotosResponse, PhotoResponse, RegisterPhotoRequest, UpdatePhotoRequest},
    model::{NewPhotoRecord, PhotoRecord, UpdatePhotoRecord},
    repository::PhotosRepository,
};
use chrono::Utc;
use shared::auth::AuthenticatedUser;
use shared::drive_client::DriveClient;
use shared::ApiError;
use std::sync::Arc;
use uuid::Uuid;

pub struct PhotosService {
    repo: Arc<PhotosRepository>,
    drive: Arc<DriveClient>,
    drive_base_url: String,
    worker_secret: String,
    http: reqwest::Client,
}

impl PhotosService {
    pub fn new(
        repo: Arc<PhotosRepository>,
        drive: Arc<DriveClient>,
        drive_base_url: String,
        worker_secret: String,
    ) -> Self {
        PhotosService {
            repo,
            drive,
            drive_base_url,
            worker_secret,
            http: reqwest::Client::new(),
        }
    }

    pub async fn register_photo(
        &self,
        user: &AuthenticatedUser,
        req: RegisterPhotoRequest,
    ) -> Result<PhotoResponse, ApiError> {
        let file = self
            .drive
            .get_file(&user.token, &req.file_id, "File not found")
            .await?;

        if file.deleted_at.is_some() {
            return Err(ApiError::not_found("File is in trash"));
        }

        let capture_date = req
            .capture_date
            .as_deref()
            .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok());

        let id = Uuid::new_v4().to_string();
        let new_photo = NewPhotoRecord {
            id: &id,
            user_id: &user.user_id,
            file_id: &req.file_id,
            is_starred: false,
            is_archived: false,
            deleted_at: None,
            capture_date,
            thumbnail: None,
            thumbnail_mime_type: None,
        };
        let photo = self.repo.insert_photo(new_photo)?;

        // Enqueue thumbnail job via drive API — failure is non-fatal.
        if let Err(e) = self.enqueue_thumbnail_job(&id, &req.file_id).await {
            tracing::warn!("Failed to enqueue thumbnail job for photo {}: {}", id, e);
        }

        Ok(self.to_response(
            photo,
            &file.name,
            file.mime_type
                .as_deref()
                .unwrap_or("application/octet-stream"),
            0,
        ))
    }

    async fn enqueue_thumbnail_job(&self, photo_id: &str, file_id: &str) -> Result<(), String> {
        let url = format!("{}/api/v1/jobs", self.drive_base_url);
        let body = serde_json::json!({
            "jobType": "thumbnail",
            "payload": { "photoId": photo_id, "fileId": file_id },
            "timeoutSecs": 30
        });
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.worker_secret))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {}", e))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("Drive jobs API returned {}", resp.status()))
        }
    }

    pub async fn list_photos(
        &self,
        user: &AuthenticatedUser,
        include_archived: bool,
        starred_only: bool,
    ) -> Result<ListPhotosResponse, ApiError> {
        let records = self
            .repo
            .list_photos(&user.user_id, include_archived, starred_only)?;
        let mut responses = Vec::with_capacity(records.len());
        for r in &records {
            let file = self
                .drive
                .get_file(&user.token, &r.file_id, "File not found")
                .await
                .ok();
            let (name, mime_type) = if let Some(f) = file {
                (
                    f.name,
                    f.mime_type
                        .unwrap_or_else(|| "application/octet-stream".to_string()),
                )
            } else {
                (
                    "Unknown".to_string(),
                    "application/octet-stream".to_string(),
                )
            };
            responses.push(self.to_response(r.clone(), &name, &mime_type, 0));
        }
        let total = responses.len();
        Ok(ListPhotosResponse {
            photos: responses,
            total,
        })
    }

    pub async fn get_photo(
        &self,
        user: &AuthenticatedUser,
        photo_id: &str,
    ) -> Result<PhotoResponse, ApiError> {
        let photo = self.repo.get_photo(photo_id)?;
        if photo.user_id != user.user_id {
            return Err(ApiError::new(403, "FORBIDDEN", "Access denied"));
        }
        let file = self
            .drive
            .get_file(&user.token, &photo.file_id, "File not found")
            .await?;
        Ok(self.to_response(
            photo,
            &file.name,
            file.mime_type
                .as_deref()
                .unwrap_or("application/octet-stream"),
            0,
        ))
    }

    pub async fn update_photo(
        &self,
        user: &AuthenticatedUser,
        photo_id: &str,
        req: UpdatePhotoRequest,
    ) -> Result<PhotoResponse, ApiError> {
        let photo = self.repo.get_photo(photo_id)?;
        if photo.user_id != user.user_id {
            return Err(ApiError::new(403, "FORBIDDEN", "Access denied"));
        }

        let changes = UpdatePhotoRecord {
            is_starred: req.is_starred,
            is_archived: req.is_archived,
            deleted_at: None,
            thumbnail: None,
            thumbnail_mime_type: None,
            updated_at: Utc::now().naive_utc(),
        };
        let updated = self.repo.update_photo(photo_id, changes)?;
        let file = self
            .drive
            .get_file(&user.token, &updated.file_id, "File not found")
            .await?;
        Ok(self.to_response(
            updated,
            &file.name,
            file.mime_type
                .as_deref()
                .unwrap_or("application/octet-stream"),
            0,
        ))
    }

    pub async fn trash_photo(
        &self,
        user: &AuthenticatedUser,
        photo_id: &str,
    ) -> Result<(), ApiError> {
        let photo = self.repo.get_photo(photo_id)?;
        if photo.user_id != user.user_id {
            return Err(ApiError::new(403, "FORBIDDEN", "Access denied"));
        }
        let changes = UpdatePhotoRecord {
            is_starred: None,
            is_archived: None,
            deleted_at: Some(Some(Utc::now().naive_utc())),
            thumbnail: None,
            thumbnail_mime_type: None,
            updated_at: Utc::now().naive_utc(),
        };
        self.repo.update_photo(photo_id, changes)?;
        Ok(())
    }

    pub async fn restore_photo(
        &self,
        user: &AuthenticatedUser,
        photo_id: &str,
    ) -> Result<PhotoResponse, ApiError> {
        let photo = self.repo.get_photo_including_deleted(photo_id)?;
        if photo.user_id != user.user_id {
            return Err(ApiError::new(403, "FORBIDDEN", "Access denied"));
        }
        if photo.deleted_at.is_none() {
            return Err(ApiError::bad_request("Photo is not in trash"));
        }
        let changes = UpdatePhotoRecord {
            is_starred: None,
            is_archived: None,
            deleted_at: Some(None),
            thumbnail: None,
            thumbnail_mime_type: None,
            updated_at: Utc::now().naive_utc(),
        };
        let updated = self.repo.update_photo(photo_id, changes)?;
        let file = self
            .drive
            .get_file(&user.token, &updated.file_id, "File not found")
            .await?;
        Ok(self.to_response(
            updated,
            &file.name,
            file.mime_type
                .as_deref()
                .unwrap_or("application/octet-stream"),
            0,
        ))
    }

    pub async fn list_trash(
        &self,
        user: &AuthenticatedUser,
    ) -> Result<ListPhotosResponse, ApiError> {
        let records = self.repo.list_trash(&user.user_id)?;
        let mut responses = Vec::with_capacity(records.len());
        for r in &records {
            let file = self
                .drive
                .get_file(&user.token, &r.file_id, "File not found")
                .await
                .ok();
            let (name, mime_type) = if let Some(f) = file {
                (
                    f.name,
                    f.mime_type
                        .unwrap_or_else(|| "application/octet-stream".to_string()),
                )
            } else {
                (
                    "Unknown".to_string(),
                    "application/octet-stream".to_string(),
                )
            };
            responses.push(self.to_response(r.clone(), &name, &mime_type, 0));
        }
        let total = responses.len();
        Ok(ListPhotosResponse {
            photos: responses,
            total,
        })
    }

    pub fn empty_trash(&self, user: &AuthenticatedUser) -> Result<(), ApiError> {
        self.repo.empty_trash(&user.user_id)?;
        Ok(())
    }

    pub fn save_thumbnail(
        &self,
        photo_id: &str,
        data: Vec<u8>,
        mime_type: String,
    ) -> Result<(), ApiError> {
        self.repo.get_photo_including_deleted(photo_id)?;
        self.repo.set_thumbnail(photo_id, data, mime_type)
    }

    pub fn get_thumbnail(&self, photo_id: &str) -> Result<Option<(Vec<u8>, String)>, ApiError> {
        self.repo.get_thumbnail(photo_id)
    }

    fn to_response(
        &self,
        photo: PhotoRecord,
        name: &str,
        mime_type: &str,
        size: i64,
    ) -> PhotoResponse {
        let thumbnail_url = if photo.thumbnail.is_some() {
            Some(format!("/api/v1/photos/{}/thumbnail", photo.id))
        } else {
            None
        };
        PhotoResponse {
            id: photo.id.clone(),
            file_id: photo.file_id.clone(),
            file_name: name.to_string(),
            mime_type: mime_type.to_string(),
            size_bytes: size,
            content_url: format!("/api/v1/drive/files/{}", photo.file_id),
            thumbnail_url,
            is_starred: photo.is_starred,
            is_archived: photo.is_archived,
            capture_date: photo.capture_date.map(|d| d.and_utc().to_rfc3339()),
            created_at: photo.created_at.and_utc().to_rfc3339(),
            updated_at: photo.updated_at.and_utc().to_rfc3339(),
        }
    }
}
