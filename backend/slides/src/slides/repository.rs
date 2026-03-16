use crate::common::ApiError;
use crate::slides::model::{NewSlideRecord, SlideRecord, UpdateSlideRecord};
use crate::schema::slides;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub struct SlidesRepository {
    pool: DbPool,
}

impl SlidesRepository {
    pub fn new(pool: DbPool) -> Self {
        SlidesRepository { pool }
    }

    fn get_conn(
        &self,
    ) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, ApiError> {
        self.pool.get().map_err(|e| {
            tracing::error!("DB pool error: {:?}", e);
            ApiError::internal("Database connection unavailable")
        })
    }

    pub fn insert_slide(&self, new_slide: NewSlideRecord) -> Result<SlideRecord, ApiError> {
        let mut conn = self.get_conn()?;
        diesel::insert_into(slides::table)
            .values(&new_slide)
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("DB insert slide error: {:?}", e);
                ApiError::internal("Database error")
            })?;
        slides::table
            .filter(slides::file_id.eq(new_slide.file_id))
            .select(SlideRecord::as_select())
            .first(&mut conn)
            .map_err(|e| {
                tracing::error!("DB query after slide insert error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    pub fn get_slide(&self, file_id: &str) -> Result<SlideRecord, ApiError> {
        let mut conn = self.get_conn()?;
        slides::table
            .filter(slides::file_id.eq(file_id))
            .select(SlideRecord::as_select())
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => ApiError::not_found("Presentation not found"),
                _ => {
                    tracing::error!("DB get slide error: {:?}", e);
                    ApiError::internal("Database error")
                }
            })
    }

    pub fn update_slide(
        &self,
        file_id: &str,
        changes: UpdateSlideRecord,
    ) -> Result<SlideRecord, ApiError> {
        let mut conn = self.get_conn()?;
        diesel::update(slides::table.filter(slides::file_id.eq(file_id)))
            .set(&changes)
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("DB update slide error: {:?}", e);
                ApiError::internal("Database error")
            })?;
        self.get_slide(file_id)
    }
}
