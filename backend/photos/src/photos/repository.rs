use crate::photos::model::{NewPhotoRecord, PhotoRecord, UpdatePhotoRecord};
use crate::schema::photos;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use shared::ApiError;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub struct PhotosRepository {
    pool: DbPool,
}

impl PhotosRepository {
    pub fn new(pool: DbPool) -> Self {
        PhotosRepository { pool }
    }

    fn get_conn(
        &self,
    ) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, ApiError> {
        self.pool.get().map_err(|e| {
            tracing::error!("DB pool error: {:?}", e);
            ApiError::internal("Database connection unavailable")
        })
    }

    pub fn insert_photo(&self, new_photo: NewPhotoRecord) -> Result<PhotoRecord, ApiError> {
        let mut conn = self.get_conn()?;
        diesel::insert_into(photos::table)
            .values(&new_photo)
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("DB insert photo error: {:?}", e);
                ApiError::internal("Database error")
            })?;
        photos::table
            .filter(photos::id.eq(new_photo.id))
            .select(PhotoRecord::as_select())
            .first(&mut conn)
            .map_err(|e| {
                tracing::error!("DB query after photo insert error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    pub fn get_photo(&self, photo_id: &str) -> Result<PhotoRecord, ApiError> {
        let mut conn = self.get_conn()?;
        photos::table
            .filter(photos::id.eq(photo_id))
            .filter(photos::deleted_at.is_null())
            .select(PhotoRecord::as_select())
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => ApiError::not_found("Photo not found"),
                _ => {
                    tracing::error!("DB get photo error: {:?}", e);
                    ApiError::internal("Database error")
                }
            })
    }

    pub fn get_photo_including_deleted(&self, photo_id: &str) -> Result<PhotoRecord, ApiError> {
        let mut conn = self.get_conn()?;
        photos::table
            .filter(photos::id.eq(photo_id))
            .select(PhotoRecord::as_select())
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => ApiError::not_found("Photo not found"),
                _ => {
                    tracing::error!("DB get photo error: {:?}", e);
                    ApiError::internal("Database error")
                }
            })
    }

    pub fn list_photos(
        &self,
        user_id: &str,
        include_archived: bool,
        starred_only: bool,
    ) -> Result<Vec<PhotoRecord>, ApiError> {
        let mut conn = self.get_conn()?;
        let mut query = photos::table
            .filter(photos::user_id.eq(user_id))
            .filter(photos::deleted_at.is_null())
            .into_boxed();

        if !include_archived {
            query = query.filter(photos::is_archived.eq(false));
        }
        if starred_only {
            query = query.filter(photos::is_starred.eq(true));
        }

        query
            .order(photos::created_at.desc())
            .select(PhotoRecord::as_select())
            .load(&mut conn)
            .map_err(|e| {
                tracing::error!("DB list photos error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    pub fn list_trash(&self, user_id: &str) -> Result<Vec<PhotoRecord>, ApiError> {
        let mut conn = self.get_conn()?;
        photos::table
            .filter(photos::user_id.eq(user_id))
            .filter(photos::deleted_at.is_not_null())
            .order(photos::deleted_at.desc())
            .select(PhotoRecord::as_select())
            .load(&mut conn)
            .map_err(|e| {
                tracing::error!("DB list trash error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    pub fn update_photo(
        &self,
        photo_id: &str,
        changes: UpdatePhotoRecord,
    ) -> Result<PhotoRecord, ApiError> {
        let mut conn = self.get_conn()?;
        diesel::update(photos::table.filter(photos::id.eq(photo_id)))
            .set(&changes)
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("DB update photo error: {:?}", e);
                ApiError::internal("Database error")
            })?;
        self.get_photo_including_deleted(photo_id)
    }

    pub fn set_thumbnail(
        &self,
        photo_id: &str,
        thumbnail: Vec<u8>,
        mime_type: String,
    ) -> Result<(), ApiError> {
        let mut conn = self.get_conn()?;
        diesel::update(photos::table.filter(photos::id.eq(photo_id)))
            .set((
                photos::thumbnail.eq(Some(thumbnail)),
                photos::thumbnail_mime_type.eq(Some(mime_type)),
                photos::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("DB set thumbnail error: {:?}", e);
                ApiError::internal("Database error")
            })?;
        Ok(())
    }

    pub fn get_thumbnail(&self, photo_id: &str) -> Result<Option<(Vec<u8>, String)>, ApiError> {
        let photo = self.get_photo(photo_id)?;
        match (photo.thumbnail, photo.thumbnail_mime_type) {
            (Some(data), Some(mime)) => Ok(Some((data, mime))),
            _ => Ok(None),
        }
    }

    pub fn delete_expired_trash(&self, before: NaiveDateTime) -> Result<usize, ApiError> {
        let mut conn = self.get_conn()?;
        diesel::delete(
            photos::table
                .filter(photos::deleted_at.is_not_null())
                .filter(photos::deleted_at.le(before)),
        )
        .execute(&mut conn)
        .map_err(|e| {
            tracing::error!("DB delete expired trash error: {:?}", e);
            ApiError::internal("Database error")
        })
    }

    pub fn empty_trash(&self, user_id: &str) -> Result<usize, ApiError> {
        let mut conn = self.get_conn()?;
        diesel::delete(
            photos::table
                .filter(photos::user_id.eq(user_id))
                .filter(photos::deleted_at.is_not_null()),
        )
        .execute(&mut conn)
        .map_err(|e| {
            tracing::error!("DB empty trash error: {:?}", e);
            ApiError::internal("Database error")
        })
    }
}
