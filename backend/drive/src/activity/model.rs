use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema::file_activity_log;

#[derive(Debug, Queryable, Selectable, Serialize, Clone)]
#[diesel(table_name = file_activity_log)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ActivityEntry {
    pub id: String,
    pub file_id: String,
    pub user_id: String,
    pub user_name: String,
    pub action: String,
    pub detail_json: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = file_activity_log)]
pub struct NewActivityEntry {
    pub id: String,
    pub file_id: String,
    pub user_id: String,
    pub user_name: String,
    pub action: String,
    pub detail_json: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}
