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

diesel::joinable!(refresh_tokens -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(users, refresh_tokens,);
