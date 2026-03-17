diesel::table! {
    photos (id) {
        id -> Text,
        user_id -> Text,
        file_id -> Text,
        is_starred -> Bool,
        is_archived -> Bool,
        deleted_at -> Nullable<Timestamp>,
        capture_date -> Nullable<Timestamp>,
        thumbnail -> Nullable<Binary>,
        thumbnail_mime_type -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    albums (id) {
        id -> Text,
        user_id -> Text,
        title -> Text,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    album_photos (album_id, photo_id) {
        album_id -> Text,
        photo_id -> Text,
        added_at -> Timestamp,
    }
}
