use crate::common::{ApiError, AuthenticatedUser};
use crate::slides::{
    dto::{CreateSlideRequest, ListSlidesResponse, SaveSlideRequest, SlideMetaResponse, SlideResponse},
    model::{NewSlideRecord, UpdateSlideRecord},
    repository::SlidesRepository,
};

fn content_urls(file_id: &str) -> (String, String) {
    (
        format!("/api/v1/drive/files/{}", file_id),
        format!("/api/v1/drive/files/{}/versions", file_id),
    )
}
use shared::drive_client::DriveClient;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// Default empty presentation: one blank title slide.
const EMPTY_SLIDES_CONTENT: &str = r#"{"slides":[{"id":"s1","background":{"type":"color","value":"\#ffffff"},"elements":[{"id":"e1","type":"text","x":10,"y":30,"w":80,"h":20,"content":"Click to add title","style":{"fontSize":40,"bold":true,"italic":false,"underline":false,"color":"\#1f2937","align":"center","fontFamily":"Inter"}},{"id":"e2","type":"text","x":15,"y":55,"w":70,"h":15,"content":"Click to add subtitle","style":{"fontSize":24,"bold":false,"italic":false,"underline":false,"color":"\#6b7280","align":"center","fontFamily":"Inter"}}],"notes":"","transition":"fade"}],"theme":{"name":"default","primaryColor":"\#4f46e5","backgroundColor":"\#ffffff","textColor":"\#1f2937","accentColor":"\#818cf8"}}"#;
const MIME_TYPE: &str = "application/x-neutrino-slide";

pub struct SlidesService {
    repo: Arc<SlidesRepository>,
    drive: Arc<DriveClient>,
}

impl SlidesService {
    pub fn new(repo: Arc<SlidesRepository>, drive: Arc<DriveClient>) -> Self {
        SlidesService { repo, drive }
    }

    pub async fn list_slides(&self, user: &AuthenticatedUser) -> Result<ListSlidesResponse, ApiError> {
        let items = self.drive.list_files(&user.token, MIME_TYPE).await?;
        let slides = items
            .into_iter()
            .map(|item| SlideMetaResponse {
                id: item.id,
                title: item.name,
                folder_id: item.folder_id,
                created_at: item.created_at.and_utc().to_rfc3339(),
                updated_at: item.updated_at.and_utc().to_rfc3339(),
            })
            .collect();
        Ok(ListSlidesResponse { slides })
    }

    pub async fn create_slide(
        &self,
        user: &AuthenticatedUser,
        req: CreateSlideRequest,
    ) -> Result<SlideResponse, ApiError> {
        let title = req.title.trim().to_string();
        if title.is_empty() {
            return Err(ApiError::bad_request("Presentation title cannot be empty"));
        }
        let id = Uuid::new_v4().to_string();
        let file = self
            .drive
            .create_file(&user.token, &id, &title, MIME_TYPE, req.folder_id.as_deref())
            .await?;
        let new_slide = NewSlideRecord { file_id: &id };
        self.repo.insert_slide(new_slide)?;

        self.drive
            .upload_content(&user.token, &id, EMPTY_SLIDES_CONTENT, "upload_slide_content")
            .await?;

        let (content_url, content_write_url) = content_urls(&id);
        Ok(SlideResponse {
            id: file.id,
            title: file.name,
            content_url,
            content_write_url,
            folder_id: file.folder_id,
            created_at: file.created_at.and_utc().to_rfc3339(),
            updated_at: file.updated_at.and_utc().to_rfc3339(),
        })
    }

    pub async fn get_slide(
        &self,
        user: &AuthenticatedUser,
        slide_id: &str,
    ) -> Result<SlideResponse, ApiError> {
        let file = self
            .drive
            .get_file(&user.token, slide_id, "Presentation not found")
            .await?;
        if file.deleted_at.is_some() {
            return Err(ApiError::not_found("Presentation is in trash"));
        }
        let (content_url, content_write_url) = content_urls(slide_id);
        Ok(SlideResponse {
            id: file.id,
            title: file.name,
            content_url,
            content_write_url,
            folder_id: file.folder_id,
            created_at: file.created_at.and_utc().to_rfc3339(),
            updated_at: file.updated_at.and_utc().to_rfc3339(),
        })
    }

    pub async fn save_slide(
        &self,
        user: &AuthenticatedUser,
        slide_id: &str,
        req: SaveSlideRequest,
    ) -> Result<SlideMetaResponse, ApiError> {
        let file = self
            .drive
            .get_file(&user.token, slide_id, "Presentation not found")
            .await?;
        match file.your_role.as_str() {
            "owner" | "editor" => {}
            _ => return Err(ApiError::new(403, "FORBIDDEN", "Edit access required")),
        }
        if file.deleted_at.is_some() {
            return Err(ApiError::not_found("Presentation is in trash"));
        }

        let new_title = if let Some(ref title) = req.title {
            let trimmed = title.trim().to_string();
            if !trimmed.is_empty() {
                self.drive.update_file_name(&user.token, slide_id, &trimmed).await?;
                trimmed
            } else {
                file.name.clone()
            }
        } else {
            file.name.clone()
        };

        let now = Utc::now().naive_utc();
        let changes = UpdateSlideRecord { updated_at: now };
        self.repo.update_slide(slide_id, changes)?;

        Ok(SlideMetaResponse {
            id: file.id,
            title: new_title,
            folder_id: file.folder_id,
            created_at: file.created_at.and_utc().to_rfc3339(),
            updated_at: now.and_utc().to_rfc3339(),
        })
    }
}
