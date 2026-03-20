use chrono::NaiveDateTime;
use diesel::prelude::*;

#[allow(dead_code)]
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::photos)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PhotoRecord {
    pub id: String,
    pub user_id: String,
    pub file_id: String,
    pub is_starred: bool,
    pub is_archived: bool,
    pub deleted_at: Option<NaiveDateTime>,
    pub capture_date: Option<NaiveDateTime>,
    pub metadata: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::photos)]
pub struct NewPhotoRecord<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub file_id: &'a str,
    pub is_starred: bool,
    pub is_archived: bool,
    pub deleted_at: Option<NaiveDateTime>,
    pub capture_date: Option<NaiveDateTime>,
    pub metadata: Option<&'a str>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = crate::schema::photos)]
pub struct UpdatePhotoRecord {
    pub is_starred: Option<bool>,
    pub is_archived: Option<bool>,
    pub deleted_at: Option<Option<NaiveDateTime>>,
    pub updated_at: NaiveDateTime,
}
