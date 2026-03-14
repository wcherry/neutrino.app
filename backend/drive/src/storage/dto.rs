use crate::storage::model::{FileRecord, FileVersionRecord};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum FileOrderField {
    Name,
    Size,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum VersionOrderField {
    VersionNumber,
    CreatedAt,
    Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum ZipEntryOrderField {
    Name,
    Size,
    CompressedSize,
    IsDir,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadataResponse {
    pub id: String,
    pub name: String,
    pub size_bytes: i64,
    pub mime_type: String,
    pub folder_id: Option<String>,
    pub is_starred: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<FileRecord> for FileMetadataResponse {
    fn from(f: FileRecord) -> Self {
        FileMetadataResponse {
            id: f.id,
            name: f.name,
            size_bytes: f.size_bytes,
            mime_type: f.mime_type,
            folder_id: f.folder_id,
            is_starred: f.is_starred,
            created_at: f.created_at,
            updated_at: f.updated_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListFilesResponse {
    pub files: Vec<FileMetadataResponse>,
    pub total: usize,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ZipEntry {
    pub name: String,
    pub size: u64,
    pub compressed_size: u64,
    pub is_dir: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ZipContentsResponse {
    pub entries: Vec<ZipEntry>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileVersionResponse {
    pub id: String,
    pub file_id: String,
    pub version_number: i32,
    pub size_bytes: i64,
    pub label: Option<String>,
    pub created_at: NaiveDateTime,
}

impl From<FileVersionRecord> for FileVersionResponse {
    fn from(v: FileVersionRecord) -> Self {
        FileVersionResponse {
            id: v.id,
            file_id: v.file_id,
            version_number: v.version_number,
            size_bytes: v.size_bytes,
            label: v.label,
            created_at: v.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListVersionsResponse {
    pub versions: Vec<FileVersionResponse>,
    pub total: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVersionLabelRequest {
    pub label: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileRequest {
    pub id: String,
    pub name: String,
    pub folder_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuotaResponse {
    pub used_bytes: i64,
    pub daily_upload_bytes: i64,
    /// `null` means no limit
    pub quota_bytes: Option<i64>,
    /// `null` means no limit
    pub daily_cap_bytes: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocFileMetadataResponse {
    pub id: String,
    pub name: String,
    pub folder_id: Option<String>,
    pub deleted_at: Option<NaiveDateTime>,
    pub your_role: String,
    pub storage_path: Option<String>,
    pub mime_type: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
