// Tables used by the docs microservice
diesel::table! {
    docs (file_id) {
        file_id -> Text,
        page_setup -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    doc_yjs_state (file_id) {
        file_id -> Text,
        state -> Binary,
        updated_at -> Timestamp,
    }
}
