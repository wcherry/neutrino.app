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

diesel::table! {
    doc_templates (id) {
        id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        is_system -> Integer,
        is_default -> Integer,
        category -> Nullable<Text>,
        content_json -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
