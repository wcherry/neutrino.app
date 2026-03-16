#[macro_export]
macro_rules! define_permission_models {
    ($table:path) => {
        #[allow(dead_code)]
        #[derive(Debug, Clone, Queryable, Selectable)]
        #[diesel(table_name = $table)]
        #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
        pub struct PermissionRecord {
            pub id: String,
            pub resource_type: String,
            pub resource_id: String,
            pub user_id: String,
            pub role: String,
            pub granted_by: String,
            pub created_at: chrono::NaiveDateTime,
            pub user_email: String,
            pub user_name: String,
        }

        #[derive(Debug, Insertable)]
        #[diesel(table_name = $table)]
        pub struct NewPermissionRecord<'a> {
            pub id: &'a str,
            pub resource_type: &'a str,
            pub resource_id: &'a str,
            pub user_id: &'a str,
            pub role: &'a str,
            pub granted_by: &'a str,
            pub user_email: &'a str,
            pub user_name: &'a str,
        }
    };
}
