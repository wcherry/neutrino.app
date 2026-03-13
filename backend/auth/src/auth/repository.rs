use crate::common::ApiError;
use crate::schema::{refresh_tokens, users};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

#[allow(dead_code)]
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    pub id: &'a str,
    pub email: &'a str,
    pub name: &'a str,
    pub password_hash: &'a str,
}

#[allow(dead_code)]
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::refresh_tokens)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RefreshToken {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::refresh_tokens)]
pub struct NewRefreshToken<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub token_hash: &'a str,
    pub expires_at: NaiveDateTime,
}

pub struct AuthRepository {
    pool: DbPool,
}

impl AuthRepository {
    pub fn new(pool: DbPool) -> Self {
        AuthRepository { pool }
    }

    pub fn find_user_by_email(&self, email_val: &str) -> Result<Option<User>, ApiError> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("DB pool error: {:?}", e);
            ApiError::internal("Database connection error")
        })?;

        let result = users::table
            .filter(users::email.eq(email_val))
            .select(User::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("DB query error: {:?}", e);
                ApiError::internal("Database query error")
            })?;

        Ok(result)
    }

    pub fn find_user_by_id(&self, user_id: &str) -> Result<Option<User>, ApiError> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("DB pool error: {:?}", e);
            ApiError::internal("Database connection error")
        })?;

        let result = users::table
            .filter(users::id.eq(user_id))
            .select(User::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("DB query error: {:?}", e);
                ApiError::internal("Database query error")
            })?;

        Ok(result)
    }

    pub fn create_user(&self, new_user: NewUser) -> Result<User, ApiError> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("DB pool error: {:?}", e);
            ApiError::internal("Database connection error")
        })?;

        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => ApiError::conflict("Email already registered"),
                _ => {
                    tracing::error!("DB insert error: {:?}", e);
                    ApiError::internal("Database error")
                }
            })?;

        let user = users::table
            .filter(users::id.eq(new_user.id))
            .select(User::as_select())
            .first(&mut conn)
            .map_err(|e| {
                tracing::error!("DB query error after insert: {:?}", e);
                ApiError::internal("Database error")
            })?;

        Ok(user)
    }

    pub fn create_refresh_token(
        &self,
        new_token: NewRefreshToken,
    ) -> Result<RefreshToken, ApiError> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("DB pool error: {:?}", e);
            ApiError::internal("Database connection error")
        })?;

        diesel::insert_into(refresh_tokens::table)
            .values(&new_token)
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("DB insert error: {:?}", e);
                ApiError::internal("Database error")
            })?;

        let token = refresh_tokens::table
            .filter(refresh_tokens::id.eq(new_token.id))
            .select(RefreshToken::as_select())
            .first(&mut conn)
            .map_err(|e| {
                tracing::error!("DB query error after insert: {:?}", e);
                ApiError::internal("Database error")
            })?;

        Ok(token)
    }

    pub fn find_refresh_token_by_hash(
        &self,
        token_hash_val: &str,
    ) -> Result<Option<RefreshToken>, ApiError> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("DB pool error: {:?}", e);
            ApiError::internal("Database connection error")
        })?;

        let result = refresh_tokens::table
            .filter(refresh_tokens::token_hash.eq(token_hash_val))
            .select(RefreshToken::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("DB query error: {:?}", e);
                ApiError::internal("Database query error")
            })?;

        Ok(result)
    }

    pub fn delete_refresh_token(&self, token_id: &str) -> Result<(), ApiError> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("DB pool error: {:?}", e);
            ApiError::internal("Database connection error")
        })?;

        diesel::delete(refresh_tokens::table.filter(refresh_tokens::id.eq(token_id)))
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("DB delete error: {:?}", e);
                ApiError::internal("Database error")
            })?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn check_db_health(&self) -> Result<(), ApiError> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("DB pool error: {:?}", e);
            ApiError::internal("Database connection error")
        })?;

        diesel::sql_query("SELECT 1")
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("DB health check error: {:?}", e);
                ApiError::internal("Database health check failed")
            })?;

        Ok(())
    }
}
