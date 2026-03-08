use crate::features::shared::{ApiError, ListQuery, OrderDirection};
use crate::features::storage::model::{FileRecord, NewFileRecord, NewUserQuota, UserQuota};
use crate::features::storage::dto::FileOrderField;
use crate::schema::{files, user_quotas};
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
