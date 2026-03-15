use crate::common::{ApiError, AuthenticatedUser};
use crate::sheets::{
    dto::{CreateSheetRequest, ListSheetsResponse, SaveSheetRequest, SheetMetaResponse, SheetResponse},
    model::{NewSheetRecord, UpdateSheetRecord},
    repository::SheetsRepository,
};
use crate::drive_client::DriveClient;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// Default empty FortuneSheet workbook: one sheet named "Sheet1".
const EMPTY_SHEET_CONTENT: &str = r#"[{"index":"0","name":"Sheet1","celldata":[],"row":100,"column":26,"order":0,"status":1,"config":{}}]"#;
const MIME_TYPE: &str = "application/x-neutrino-sheet";

pub struct SheetsService {
    repo: Arc<SheetsRepository>,
    drive: Arc<DriveClient>,
}

impl SheetsService {
    pub fn new(repo: Arc<SheetsRepository>, drive: Arc<DriveClient>) -> Self {
        SheetsService { repo, drive }
    }

    pub async fn list_sheets(&self, user: &AuthenticatedUser) -> Result<ListSheetsResponse, ApiError> {
        let items = self.drive.list_sheets(&user.token).await?;
        let sheets = items
            .into_iter()
            .map(|item| SheetMetaResponse {
                id: item.id,
                title: item.name,
                folder_id: item.folder_id,
                created_at: item.created_at.and_utc().to_rfc3339(),
                updated_at: item.updated_at.and_utc().to_rfc3339(),
            })
            .collect();
        Ok(ListSheetsResponse { sheets })
    }

    pub async fn create_sheet(
        &self,
        user: &AuthenticatedUser,
        req: CreateSheetRequest,
    ) -> Result<SheetResponse, ApiError> {
        let title = req.title.trim().to_string();
        if title.is_empty() {
            return Err(ApiError::bad_request("Spreadsheet title cannot be empty"));
        }
        let id = Uuid::new_v4().to_string();
        let file = self
            .drive
            .register_sheet(&user.token, &id, &title, &MIME_TYPE, req.folder_id.as_deref())
            .await?;
        let new_sheet = NewSheetRecord { file_id: &id };
        self.repo.insert_sheet(new_sheet)?;

        self.drive
            .upload_sheet_content(&user.token, &id, EMPTY_SHEET_CONTENT)
            .await?;

        Ok(SheetResponse {
            id: file.id,
            title: file.name,
            content: EMPTY_SHEET_CONTENT.to_string(),
            folder_id: file.folder_id,
            created_at: file.created_at.and_utc().to_rfc3339(),
            updated_at: file.updated_at.and_utc().to_rfc3339(),
        })
    }

    pub async fn get_sheet(
        &self,
        user: &AuthenticatedUser,
        sheet_id: &str,
    ) -> Result<SheetResponse, ApiError> {
        let file = self.drive.get_file(&user.token, sheet_id).await?;
        if file.deleted_at.is_some() {
            return Err(ApiError::not_found("Spreadsheet is in trash"));
        }
        let content = match file.storage_path {
            Some(_) => self.drive.get_sheet_content(&user.token, sheet_id).await?,
            None => EMPTY_SHEET_CONTENT.to_string(),
        };
        Ok(SheetResponse {
            id: file.id,
            title: file.name,
            content,
            folder_id: file.folder_id,
            created_at: file.created_at.and_utc().to_rfc3339(),
            updated_at: file.updated_at.and_utc().to_rfc3339(),
        })
    }

    pub async fn save_sheet(
        &self,
        user: &AuthenticatedUser,
        sheet_id: &str,
        req: SaveSheetRequest,
    ) -> Result<SheetMetaResponse, ApiError> {
        let file = self.drive.get_file(&user.token, sheet_id).await?;
        match file.your_role.as_str() {
            "owner" | "editor" => {}
            _ => return Err(ApiError::new(403, "FORBIDDEN", "Edit access required")),
        }
        if file.deleted_at.is_some() {
            return Err(ApiError::not_found("Spreadsheet is in trash"));
        }

        let new_title = if let Some(ref title) = req.title {
            let trimmed = title.trim().to_string();
            if !trimmed.is_empty() {
                self.drive.update_file_name(&user.token, sheet_id, &trimmed).await?;
                trimmed
            } else {
                file.name.clone()
            }
        } else {
            file.name.clone()
        };

        if let Some(ref content) = req.content {
            self.drive
                .upload_sheet_content(&user.token, sheet_id, content)
                .await?;
        }

        let now = Utc::now().naive_utc();
        let changes = UpdateSheetRecord { updated_at: now };
        self.repo.update_sheet(sheet_id, changes)?;

        Ok(SheetMetaResponse {
            id: file.id,
            title: new_title,
            folder_id: file.folder_id,
            created_at: file.created_at.and_utc().to_rfc3339(),
            updated_at: now.and_utc().to_rfc3339(),
        })
    }
}
