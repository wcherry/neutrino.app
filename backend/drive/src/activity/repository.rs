use crate::activity::model::{ActivityEntry, NewActivityEntry};
use crate::schema::file_activity_log;
use crate::common::ApiError;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub struct ActivityRepository {
    pool: DbPool,
}

impl ActivityRepository {
    pub fn new(pool: DbPool) -> Self {
        ActivityRepository { pool }
    }

    fn get_conn(
        &self,
    ) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, ApiError> {
        self.pool.get().map_err(|e| {
            tracing::error!("DB pool error: {:?}", e);
            ApiError::internal("Database connection unavailable")
        })
    }

    pub fn insert_entry(&self, entry: &NewActivityEntry) -> Result<(), ApiError> {
        let mut conn = self.get_conn()?;
        diesel::insert_into(file_activity_log::table)
            .values(entry)
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("DB insert activity entry error: {:?}", e);
                ApiError::internal("Database error")
            })?;
        Ok(())
    }

    pub fn list_for_file(
        &self,
        file_id: &str,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<ActivityEntry>, i64), ApiError> {
        let mut conn = self.get_conn()?;
        let offset = (page - 1) * page_size;

        let items = file_activity_log::table
            .filter(file_activity_log::file_id.eq(file_id))
            .order(file_activity_log::created_at.desc())
            .limit(page_size)
            .offset(offset)
            .select(ActivityEntry::as_select())
            .load(&mut conn)
            .map_err(|e| {
                tracing::error!("DB list activity error: {:?}", e);
                ApiError::internal("Database error")
            })?;

        let total: i64 = file_activity_log::table
            .filter(file_activity_log::file_id.eq(file_id))
            .count()
            .get_result(&mut conn)
            .map_err(|e| {
                tracing::error!("DB count activity error: {:?}", e);
                ApiError::internal("Database error")
            })?;

        Ok((items, total))
    }
}
