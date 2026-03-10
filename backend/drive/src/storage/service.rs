use crate::shared::{ApiError, ListQuery};
use crate::storage::{
    dto::{FileMetadataResponse, FileOrderField, ListFilesResponse, QuotaResponse},
    model::NewFileRecord,
    repository::StorageRepository,
    store::LocalFileStore,
};
use chrono::Utc;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

pub struct StorageService {
    repo: Arc<StorageRepository>,
    store: Arc<LocalFileStore>,
}

impl StorageService {
    pub fn new(repo: Arc<StorageRepository>, store: Arc<LocalFileStore>) -> Self {
        StorageService { repo, store }
    }

    /// Called after a file has been streamed to `temp_path`.
    /// Enforces per-user quota and daily cap, then commits the upload.
    pub fn finalize_upload(
        &self,
        user_id: &str,
        temp_path: &Path,
        file_name: &str,
        mime_type: &str,
        size_bytes: i64,
    ) -> Result<FileMetadataResponse, ApiError> {
        let quota = self.repo.get_or_create_quota(user_id)?;

        let now = Utc::now().naive_utc();
        let today = now.date();
        let reset_daily = quota.daily_reset_at.date() < today;
        let current_daily = if reset_daily { 0 } else { quota.daily_upload_bytes };

        // Enforce quota limits if set
        if let Some(limit) = quota.quota_bytes {
            if quota.used_bytes + size_bytes > limit {
                return Err(ApiError::new(413, "QUOTA_EXCEEDED", "Storage quota exceeded"));
            }
        }
        if let Some(cap) = quota.daily_cap_bytes {
            if current_daily + size_bytes > cap {
                return Err(ApiError::new(
                    429,
                    "DAILY_LIMIT_EXCEEDED",
                    "Daily upload limit exceeded",
                ));
            }
        }

        let file_id = Uuid::new_v4().to_string();
        let final_path = self.store.file_path(user_id, &file_id);

        std::fs::rename(temp_path, &final_path).map_err(|e| {
            log::error!("Failed to move temp file to final path: {:?}", e);
            ApiError::internal("Failed to save uploaded file")
        })?;

        let storage_path = final_path.to_string_lossy().to_string();

        let new_file = NewFileRecord {
            id: &file_id,
            user_id,
            name: file_name,
            size_bytes,
            mime_type,
            storage_path: &storage_path,
        };

        let file = self.repo.insert_file(new_file).inspect_err(|e| {
            let _ = std::fs::remove_file(&final_path);
        })?;

        if let Err(e) = self.repo.update_quota_after_upload(
            user_id,
            size_bytes,
            quota.used_bytes,
            quota.daily_upload_bytes,
            now,
            reset_daily,
        ) {
            log::error!("Quota update failed for user {}: {:?}", user_id, e);
        }

        Ok(FileMetadataResponse::from(file))
    }

    pub fn list_files(
        &self,
        user_id: &str,
        query: &ListQuery<FileOrderField>,
    ) -> Result<ListFilesResponse, ApiError> {
        let files = self.repo.list_files_by_user(user_id, query)?;
        let total = files.len();
        Ok(ListFilesResponse {
            files: files.into_iter().map(FileMetadataResponse::from).collect(),
            total,
            limit: query.limit,
            offset: query.offset,
        })
    }

    pub fn get_file_metadata(
        &self,
        user_id: &str,
        file_id: &str,
    ) -> Result<FileMetadataResponse, ApiError> {
        let file = self
            .repo
            .find_file(file_id, user_id)?
            .ok_or_else(|| ApiError::not_found("File not found"))?;
        Ok(FileMetadataResponse::from(file))
    }

    /// Returns the filesystem path for serving the file.
    pub fn resolve_file_path(
        &self,
        user_id: &str,
        file_id: &str,
    ) -> Result<(std::path::PathBuf, String, String), ApiError> {
        let file = self
            .repo
            .find_file(file_id, user_id)?
            .ok_or_else(|| ApiError::not_found("File not found"))?;
        Ok((
            std::path::PathBuf::from(&file.storage_path),
            file.mime_type,
            file.name,
        ))
    }

    pub fn get_quota(&self, user_id: &str) -> Result<QuotaResponse, ApiError> {
        let quota = self.repo.get_or_create_quota(user_id)?;
        Ok(QuotaResponse {
            used_bytes: quota.used_bytes,
            daily_upload_bytes: quota.daily_upload_bytes,
            quota_bytes: quota.quota_bytes,
            daily_cap_bytes: quota.daily_cap_bytes,
        })
    }

    pub fn store(&self) -> &LocalFileStore {
        &self.store
    }
}
