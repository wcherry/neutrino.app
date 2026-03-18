use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterPhotoRequest {
    pub file_id: String,
    pub capture_date: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePhotoRequest {
    pub is_starred: Option<bool>,
    pub is_archived: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PhotoResponse {
    pub id: String,
    pub file_id: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    /// URL to download/preview the photo via drive API
    pub content_url: String,
    /// Base64-encoded thumbnail string (null if not yet generated)
    pub thumbnail: Option<String>,
    pub thumbnail_mime_type: Option<String>,
    pub is_starred: bool,
    pub is_archived: bool,
    pub capture_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// Extracted image metadata (dimensions, EXIF), null until the worker has processed it.
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListPhotosResponse {
    pub photos: Vec<PhotoResponse>,
    pub total: usize,
}

