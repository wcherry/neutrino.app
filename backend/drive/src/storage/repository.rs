use crate::shared::{ApiError, ListQuery, OrderDirection};
use crate::storage::model::{
    FileRecord, FileVersionRecord, NewFileRecord, NewFileVersionRecord, NewUserQuota,
    UpdateFileContent, UserQuota,
};
use crate::storage::dto::FileOrderField;
use crate::schema::{file_versions, files, user_quotas};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub struct StorageRepository {
    pool: DbPool,
}

impl StorageRepository {
    pub fn new(pool: DbPool) -> Self {
        StorageRepository { pool }
    }

    pub fn insert_file(&self, new_file: NewFileRecord) -> Result<FileRecord, ApiError> {
        let mut conn = self.get_conn()?;

        diesel::insert_into(files::table)
            .values(&new_file)
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("DB insert file error: {:?}", e);
                ApiError::internal("Database error")
            })?;

        files::table
            .filter(files::id.eq(new_file.id))
            .select(FileRecord::as_select())
            .first(&mut conn)
            .map_err(|e| {
                log::error!("DB query after insert error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    pub fn list_files_by_user(
        &self,
        user_id: &str,
        query: &ListQuery<FileOrderField>,
    ) -> Result<Vec<FileRecord>, ApiError> {
        let mut conn = self.get_conn()?;

        let order_by = query.order_by.unwrap_or(FileOrderField::CreatedAt);
        let direction = query.direction.unwrap_or(OrderDirection::Desc);

        macro_rules! load_ordered {
            ($col:expr) => {{
                let base = files::table
                    .filter(files::user_id.eq(user_id))
                    .filter(files::is_trashed.eq(false))
                    .select(FileRecord::as_select())
                    .limit(query.limit)
                    .offset(query.offset);
                match direction {
                    OrderDirection::Asc => base.order($col.asc()).load(&mut conn),
                    OrderDirection::Desc => base.order($col.desc()).load(&mut conn),
                }
            }};
        }

        let result = match order_by {
            FileOrderField::Name => load_ordered!(files::name),
            FileOrderField::Size => load_ordered!(files::size_bytes),
            FileOrderField::CreatedAt => load_ordered!(files::created_at),
            FileOrderField::UpdatedAt => load_ordered!(files::updated_at),
        };

        result.map_err(|e| {
            log::error!("DB list files error: {:?}", e);
            ApiError::internal("Database error")
        })
    }

    pub fn find_file(
        &self,
        file_id: &str,
        user_id: &str,
    ) -> Result<Option<FileRecord>, ApiError> {
        let mut conn = self.get_conn()?;

        files::table
            .filter(files::id.eq(file_id))
            .filter(files::user_id.eq(user_id))
            .select(FileRecord::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| {
                log::error!("DB find file error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    pub fn get_or_create_quota(&self, user_id: &str) -> Result<UserQuota, ApiError> {
        let mut conn = self.get_conn()?;

        let existing = user_quotas::table
            .filter(user_quotas::user_id.eq(user_id))
            .select(UserQuota::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| {
                log::error!("DB get quota error: {:?}", e);
                ApiError::internal("Database error")
            })?;

        if let Some(quota) = existing {
            return Ok(quota);
        }

        diesel::insert_into(user_quotas::table)
            .values(NewUserQuota { user_id })
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("DB create quota error: {:?}", e);
                ApiError::internal("Database error")
            })?;

        user_quotas::table
            .filter(user_quotas::user_id.eq(user_id))
            .select(UserQuota::as_select())
            .first(&mut conn)
            .map_err(|e| {
                log::error!("DB get quota after create error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    pub fn update_quota_after_upload(
        &self,
        user_id: &str,
        file_size: i64,
        prev_used: i64,
        prev_daily: i64,
        new_daily_reset: NaiveDateTime,
        reset_daily: bool,
    ) -> Result<(), ApiError> {
        let mut conn = self.get_conn()?;

        let new_daily = if reset_daily { file_size } else { prev_daily + file_size };

        diesel::update(user_quotas::table.filter(user_quotas::user_id.eq(user_id)))
            .set((
                user_quotas::used_bytes.eq(prev_used + file_size),
                user_quotas::daily_upload_bytes.eq(new_daily),
                user_quotas::daily_reset_at.eq(new_daily_reset),
            ))
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("DB update quota error: {:?}", e);
                ApiError::internal("Database error")
            })?;

        Ok(())
    }

    pub fn update_file_content(
        &self,
        file_id: &str,
        user_id: &str,
        changeset: UpdateFileContent,
    ) -> Result<FileRecord, ApiError> {
        let mut conn = self.get_conn()?;

        diesel::update(
            files::table
                .filter(files::id.eq(file_id))
                .filter(files::user_id.eq(user_id)),
        )
        .set(&changeset)
        .execute(&mut conn)
        .map_err(|e| {
            log::error!("DB update file content error: {:?}", e);
            ApiError::internal("Database error")
        })?;

        files::table
            .filter(files::id.eq(file_id))
            .select(FileRecord::as_select())
            .first(&mut conn)
            .map_err(|e| {
                log::error!("DB fetch updated file error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    // ── Version methods ────────────────────────────────────────────────────────

    pub fn insert_version(
        &self,
        new_version: NewFileVersionRecord,
    ) -> Result<FileVersionRecord, ApiError> {
        let mut conn = self.get_conn()?;

        diesel::insert_into(file_versions::table)
            .values(&new_version)
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("DB insert version error: {:?}", e);
                ApiError::internal("Database error")
            })?;

        file_versions::table
            .filter(file_versions::id.eq(new_version.id))
            .select(FileVersionRecord::as_select())
            .first(&mut conn)
            .map_err(|e| {
                log::error!("DB query after version insert error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    pub fn list_versions(&self, file_id: &str) -> Result<Vec<FileVersionRecord>, ApiError> {
        let mut conn = self.get_conn()?;

        file_versions::table
            .filter(file_versions::file_id.eq(file_id))
            .select(FileVersionRecord::as_select())
            .order(file_versions::version_number.desc())
            .load(&mut conn)
            .map_err(|e| {
                log::error!("DB list versions error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    pub fn find_version(
        &self,
        version_id: &str,
        file_id: &str,
        user_id: &str,
    ) -> Result<Option<FileVersionRecord>, ApiError> {
        let mut conn = self.get_conn()?;

        file_versions::table
            .filter(file_versions::id.eq(version_id))
            .filter(file_versions::file_id.eq(file_id))
            .filter(file_versions::user_id.eq(user_id))
            .select(FileVersionRecord::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| {
                log::error!("DB find version error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    pub fn count_versions(&self, file_id: &str) -> Result<i64, ApiError> {
        let mut conn = self.get_conn()?;

        file_versions::table
            .filter(file_versions::file_id.eq(file_id))
            .count()
            .get_result(&mut conn)
            .map_err(|e| {
                log::error!("DB count versions error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    pub fn max_version_number(&self, file_id: &str) -> Result<i32, ApiError> {
        use diesel::dsl::max;
        let mut conn = self.get_conn()?;

        file_versions::table
            .filter(file_versions::file_id.eq(file_id))
            .select(max(file_versions::version_number))
            .first::<Option<i32>>(&mut conn)
            .map(|v| v.unwrap_or(0))
            .map_err(|e| {
                log::error!("DB max version number error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    pub fn update_version_label(
        &self,
        version_id: &str,
        file_id: &str,
        user_id: &str,
        label: Option<String>,
    ) -> Result<FileVersionRecord, ApiError> {
        let mut conn = self.get_conn()?;

        diesel::update(
            file_versions::table
                .filter(file_versions::id.eq(version_id))
                .filter(file_versions::file_id.eq(file_id))
                .filter(file_versions::user_id.eq(user_id)),
        )
        .set(file_versions::label.eq(&label))
        .execute(&mut conn)
        .map_err(|e| {
            log::error!("DB update version label error: {:?}", e);
            ApiError::internal("Database error")
        })?;

        file_versions::table
            .filter(file_versions::id.eq(version_id))
            .select(FileVersionRecord::as_select())
            .first(&mut conn)
            .map_err(|e| {
                log::error!("DB fetch updated version error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    pub fn delete_version(
        &self,
        version_id: &str,
        file_id: &str,
        user_id: &str,
    ) -> Result<Option<String>, ApiError> {
        let mut conn = self.get_conn()?;

        let version = file_versions::table
            .filter(file_versions::id.eq(version_id))
            .filter(file_versions::file_id.eq(file_id))
            .filter(file_versions::user_id.eq(user_id))
            .select(FileVersionRecord::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| {
                log::error!("DB find version for delete error: {:?}", e);
                ApiError::internal("Database error")
            })?;

        let Some(version) = version else {
            return Ok(None);
        };

        diesel::delete(file_versions::table.filter(file_versions::id.eq(version_id)))
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("DB delete version error: {:?}", e);
                ApiError::internal("Database error")
            })?;

        Ok(Some(version.storage_path))
    }

    /// Deletes the oldest version for a file and returns its storage_path for disk cleanup.
    pub fn delete_oldest_version(&self, file_id: &str) -> Result<Option<String>, ApiError> {
        let mut conn = self.get_conn()?;

        let oldest = file_versions::table
            .filter(file_versions::file_id.eq(file_id))
            .select(FileVersionRecord::as_select())
            .order(file_versions::version_number.asc())
            .first(&mut conn)
            .optional()
            .map_err(|e| {
                log::error!("DB find oldest version error: {:?}", e);
                ApiError::internal("Database error")
            })?;

        let Some(version) = oldest else {
            return Ok(None);
        };

        diesel::delete(file_versions::table.filter(file_versions::id.eq(&version.id)))
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("DB delete oldest version error: {:?}", e);
                ApiError::internal("Database error")
            })?;

        Ok(Some(version.storage_path))
    }

    fn get_conn(
        &self,
    ) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, ApiError>
    {
        self.pool.get().map_err(|e| {
            log::error!("DB pool error: {:?}", e);
            ApiError::internal("Database connection error")
        })
    }
}
