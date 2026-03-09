use chrono::NaiveDateTime;
use diesel::prelude::*;

#[allow(dead_code)]
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::files)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct FileRecord {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub size_bytes: i64,
    pub mime_type: String,
    pub storage_path: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub folder_id: Option<String>,
    pub is_starred: bool,
    pub is_trashed: bool,
    pub trashed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::files)]
pub struct NewFileRecord<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub name: &'a str,
    pub size_bytes: i64,
    pub mime_type: &'a str,
    pub storage_path: &'a str,
}

#[allow(dead_code)]
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::user_quotas)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserQuota {
    pub user_id: String,
    pub used_bytes: i64,
    pub daily_upload_bytes: i64,
    pub daily_reset_at: NaiveDateTime,
    pub quota_bytes: Option<i64>,
    pub daily_cap_bytes: Option<i64>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::user_quotas)]
pub struct NewUserQuota<'a> {
    pub user_id: &'a str,
}
