// @generated automatically by Diesel CLI.
// Manually maintained to match migrations.

diesel::table! {
    users (id) {
        id -> Text,
        email -> Text,
        name -> Text,
        password_hash -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    refresh_tokens (id) {
        id -> Text,
        user_id -> Text,
        token_hash -> Text,
        expires_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    files (id) {
        id -> Text,
        user_id -> Text,
        name -> Text,
        size_bytes -> BigInt,
        mime_type -> Text,
        storage_path -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    user_quotas (user_id) {
        user_id -> Text,
        used_bytes -> BigInt,
        daily_upload_bytes -> BigInt,
        daily_reset_at -> Timestamp,
        quota_bytes -> Nullable<BigInt>,
        daily_cap_bytes -> Nullable<BigInt>,
    }
}

diesel::joinable!(refresh_tokens -> users (user_id));
diesel::joinable!(files -> users (user_id));
diesel::joinable!(user_quotas -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(users, refresh_tokens, files, user_quotas,);
