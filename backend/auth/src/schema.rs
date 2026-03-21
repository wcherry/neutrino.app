// @generated automatically by Diesel CLI.
// Manually maintained to match migrations.

diesel::table! {
    users (id) {
        id -> Text,
        email -> Text,
        name -> Text,
        password_hash -> Text,
        created_at -> Timestamp,
        role -> Text,
        totp_secret -> Nullable<Text>,
        totp_enabled -> Integer,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    refresh_tokens (id) {
        id -> Text,
        user_id -> Text,
        token_hash -> Text,
        expires_at -> Timestamp,
        created_at -> Timestamp,
        device_name -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        ip_address -> Nullable<Text>,
        last_used_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    totp_backup_codes (id) {
        id -> Text,
        user_id -> Text,
        code_hash -> Text,
        used_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::joinable!(refresh_tokens -> users (user_id));
diesel::joinable!(totp_backup_codes -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    refresh_tokens,
    totp_backup_codes,
);
