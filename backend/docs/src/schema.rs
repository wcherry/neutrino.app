// Tables used by the docs microservice
diesel::table! {
    docs (file_id) {
        file_id -> Text,
        page_setup -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

// diesel::allow_tables_to_appear_in_same_query!(
//     docs,
// );
