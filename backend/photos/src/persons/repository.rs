use crate::faces::model::FaceRecord;
use crate::persons::model::{NewPersonRecord, PersonRecord};
use crate::schema::{faces, persons};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;
use shared::ApiError;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub struct PersonsRepository {
    pool: DbPool,
}

impl PersonsRepository {
    pub fn new(pool: DbPool) -> Self {
        PersonsRepository { pool }
    }

    fn get_conn(
        &self,
    ) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, ApiError> {
        self.pool.get().map_err(|e| {
            tracing::error!("DB pool error: {:?}", e);
            ApiError::internal("Database connection unavailable")
        })
    }

    pub fn list_persons_for_user(&self, user_id: &str) -> Result<Vec<PersonRecord>, ApiError> {
        let mut conn = self.get_conn()?;
        persons::table
            .filter(persons::user_id.eq(user_id))
            .order(persons::face_count.desc())
            .select(PersonRecord::as_select())
            .load(&mut conn)
            .map_err(|e| {
                tracing::error!("DB list persons error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    pub fn get_person(&self, person_id: &str) -> Result<PersonRecord, ApiError> {
        let mut conn = self.get_conn()?;
        persons::table
            .filter(persons::id.eq(person_id))
            .select(PersonRecord::as_select())
            .first(&mut conn)
            .map_err(|e| {
                tracing::error!("DB get person error: {:?}", e);
                ApiError::not_found("Person not found")
            })
    }

    /// Return all distinct user_ids that have at least one face with an embedding.
    pub fn list_users_with_face_embeddings(&self) -> Result<Vec<String>, ApiError> {
        use crate::schema::photos;
        let mut conn = self.get_conn()?;
        faces::table
            .inner_join(photos::table.on(faces::photo_id.eq(photos::id)))
            .filter(faces::embedding.is_not_null())
            .select(photos::user_id)
            .distinct()
            .load::<String>(&mut conn)
            .map_err(|e| {
                tracing::error!("DB list users with faces error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    /// Return all face records (with embeddings) for photos owned by a user.
    pub fn list_face_embeddings_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<FaceRecord>, ApiError> {
        use crate::schema::photos;
        let mut conn = self.get_conn()?;
        faces::table
            .inner_join(photos::table.on(faces::photo_id.eq(photos::id)))
            .filter(photos::user_id.eq(user_id))
            .filter(faces::embedding.is_not_null())
            .select(FaceRecord::as_select())
            .load(&mut conn)
            .map_err(|e| {
                tracing::error!("DB list face embeddings error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    /// Return face records for a set of persons in one query (avoids N+1).
    pub fn list_faces_for_persons(&self, person_ids: &[String]) -> Result<Vec<FaceRecord>, ApiError> {
        let mut conn = self.get_conn()?;
        faces::table
            .filter(faces::person_id.eq_any(person_ids))
            .select(FaceRecord::as_select())
            .load(&mut conn)
            .map_err(|e| {
                tracing::error!("DB list faces for persons error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    /// Return face records for photos of a specific person (for the People detail view).
    pub fn list_faces_for_person(&self, person_id: &str) -> Result<Vec<FaceRecord>, ApiError> {
        let mut conn = self.get_conn()?;
        faces::table
            .filter(faces::person_id.eq(person_id))
            .select(FaceRecord::as_select())
            .load(&mut conn)
            .map_err(|e| {
                tracing::error!("DB list faces for person error: {:?}", e);
                ApiError::internal("Database error")
            })
    }

    /// Atomically replace all persons for a user with the new cluster set.
    /// Also updates faces.person_id assignments.
    pub fn apply_clusters(
        &self,
        user_id: &str,
        clusters: &[(String, Vec<String>, Option<String>, Option<String>, Option<String>)],
        // ^ (person_id, face_ids, cover_face_id, cover_thumbnail, cover_thumbnail_mime_type)
        now: NaiveDateTime,
    ) -> Result<(), ApiError> {
        let mut conn = self.get_conn()?;
        conn.transaction::<(), diesel::result::Error, _>(|conn| {
            // 1. Clear existing person assignments on faces for this user.
            let photo_ids: Vec<String> = {
                use crate::schema::photos;
                photos::table
                    .filter(photos::user_id.eq(user_id))
                    .select(photos::id)
                    .load::<String>(conn)?
            };
            if !photo_ids.is_empty() {
                diesel::update(faces::table.filter(faces::photo_id.eq_any(&photo_ids)))
                    .set(faces::person_id.eq::<Option<&str>>(None))
                    .execute(conn)?;
            }

            // 2. Delete all existing persons for this user.
            diesel::delete(persons::table.filter(persons::user_id.eq(user_id)))
                .execute(conn)?;

            // 3. Insert new persons and assign face person_ids.
            for (person_id, face_ids, cover_face_id, cover_thumb, cover_thumb_mime) in clusters {
                let new_person = NewPersonRecord {
                    id: person_id,
                    user_id,
                    cover_face_id: cover_face_id.as_deref(),
                    cover_thumbnail: cover_thumb.as_deref(),
                    cover_thumbnail_mime_type: cover_thumb_mime.as_deref(),
                    face_count: face_ids.len() as i32,
                    created_at: now,
                    updated_at: now,
                };
                diesel::insert_into(persons::table)
                    .values(&new_person)
                    .execute(conn)?;

                if !face_ids.is_empty() {
                    diesel::update(faces::table.filter(faces::id.eq_any(face_ids)))
                        .set(faces::person_id.eq(person_id.as_str()))
                        .execute(conn)?;
                }
            }

            Ok(())
        })
        .map_err(|e| {
            tracing::error!("DB apply clusters error: {:?}", e);
            ApiError::internal("Database error")
        })
    }
}
