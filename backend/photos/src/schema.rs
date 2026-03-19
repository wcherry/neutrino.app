diesel::table! {
    photos (id) {
        id -> Text,
        user_id -> Text,
        file_id -> Text,
        is_starred -> Bool,
        is_archived -> Bool,
        deleted_at -> Nullable<Timestamp>,
        capture_date -> Nullable<Timestamp>,
        thumbnail -> Nullable<Text>,
        thumbnail_mime_type -> Nullable<Text>,
        metadata -> Nullable<Text>,
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
        is_auto -> Bool,
        person_id -> Nullable<Text>,
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

diesel::table! {
    faces (id) {
        id -> Text,
        photo_id -> Text,
        bounding_box -> Text,
        thumbnail -> Nullable<Text>,
        thumbnail_mime_type -> Nullable<Text>,
        person_id -> Nullable<Text>,
        embedding -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    face_suggestions (id) {
        id -> Text,
        face_id -> Text,
        person_id -> Text,
        confidence -> Float,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    training_signals (id) {
        id -> Text,
        user_id -> Text,
        face_id -> Text,
        person_id -> Text,
        action -> Text,
        processed -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    user_recognition_thresholds (user_id) {
        user_id -> Text,
        auto_tag_threshold -> Float,
        suggest_threshold -> Float,
        total_accepts -> Integer,
        total_rejects -> Integer,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(faces -> photos (photo_id));
diesel::joinable!(face_suggestions -> faces (face_id));
diesel::allow_tables_to_appear_in_same_query!(face_suggestions, faces, photos);

diesel::table! {
    persons (id) {
        id -> Text,
        user_id -> Text,
        cover_face_id -> Nullable<Text>,
        cover_thumbnail -> Nullable<Text>,
        cover_thumbnail_mime_type -> Nullable<Text>,
        face_count -> Integer,
        name -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
