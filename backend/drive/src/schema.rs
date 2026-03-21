// @generated automatically by Diesel CLI.
// Manually maintained to match migrations.

diesel::table! {
    folders (id) {
        id -> Text,
        user_id -> Text,
        parent_id -> Nullable<Text>,
        name -> Text,
        is_starred -> Bool,
        color -> Nullable<Text>,
        deleted_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        // Added in migration 021
        starred_at -> Nullable<Timestamp>,
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
        // Added in migration 005
        folder_id -> Nullable<Text>,
        is_starred -> Bool,
        deleted_at -> Nullable<Timestamp>,
        // Added in migration 020
        cover_thumbnail -> Nullable<Text>,
        cover_thumbnail_mime_type -> Nullable<Text>,
        // Added in migration 021
        starred_at -> Nullable<Timestamp>,
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

diesel::table! {
    shortcuts (id) {
        id -> Text,
        user_id -> Text,
        target_file_id -> Text,
        folder_id -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    file_versions (id) {
        id -> Text,
        file_id -> Text,
        user_id -> Text,
        version_number -> Integer,
        size_bytes -> BigInt,
        storage_path -> Text,
        label -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    share_links (id) {
        id -> Text,
        resource_type -> Text,
        resource_id -> Text,
        token -> Text,
        visibility -> Text,
        role -> Text,
        expires_at -> Nullable<Timestamp>,
        is_active -> Bool,
        created_by -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    permissions (id) {
        id -> Text,
        resource_type -> Text,
        resource_id -> Text,
        user_id -> Text,
        role -> Text,
        granted_by -> Text,
        created_at -> Timestamp,
        // Added in migration 010
        user_email -> Text,
        user_name -> Text,
    }
}

diesel::table! {
    access_requests (id) {
        id -> Text,
        resource_type -> Text,
        resource_id -> Text,
        requester_id -> Text,
        requester_email -> Text,
        requester_name -> Text,
        message -> Nullable<Text>,
        requested_role -> Text,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    irm_policies (id) {
        id -> Text,
        resource_type -> Text,
        resource_id -> Text,
        restrict_download_viewer -> Bool,
        restrict_download_commenter -> Bool,
        restrict_download_editor -> Bool,
        restrict_print_copy_viewer -> Bool,
        restrict_print_copy_commenter -> Bool,
        restrict_print_copy_editor -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    workspace_settings (id) {
        id -> Text,
        allowed_domain -> Nullable<Text>,
        restrict_shares_to_domain -> Bool,
        block_external_link_sharing -> Bool,
        domain_only_links -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    worker_jobs (id) {
        id -> Text,
        job_type -> Text,
        payload -> Text,
        status -> Text,
        error_message -> Nullable<Text>,
        worker_id -> Nullable<Text>,
        timeout_secs -> Integer,
        started_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    worker_registrations (id) {
        id -> Text,
        callback_url -> Text,
        registered_at -> Timestamp,
        last_seen_at -> Timestamp,
    }
}

diesel::table! {
    doc_suggestions (id) {
        id -> Text,
        file_id -> Text,
        user_id -> Text,
        user_name -> Text,
        content_json -> Text,
        status -> Text,
        created_at -> Timestamp,
        resolved_at -> Nullable<Timestamp>,
        resolved_by -> Nullable<Text>,
    }
}

diesel::table! {
    comments (id) {
        id -> Text,
        file_id -> Text,
        user_id -> Text,
        user_name -> Text,
        anchor_json -> Nullable<Text>,
        body -> Text,
        status -> Text,
        assignee_id -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        resolved_at -> Nullable<Timestamp>,
        resolved_by -> Nullable<Text>,
    }
}

diesel::table! {
    comment_replies (id) {
        id -> Text,
        comment_id -> Text,
        user_id -> Text,
        user_name -> Text,
        body -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    notifications (id) {
        id -> Text,
        recipient_id -> Text,
        event_type -> Text,
        payload -> Text,
        is_read -> Integer,
        email_sent -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    file_activity_log (id) {
        id -> Text,
        file_id -> Text,
        user_id -> Text,
        user_name -> Text,
        action -> Text,
        detail_json -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    file_content_index (file_id) {
        file_id -> Text,
        user_id -> Text,
        indexed_at -> Timestamp,
        text_content -> Text,
    }
}

diesel::table! {
    file_access_scores (file_id) {
        file_id -> Text,
        user_id -> Text,
        score -> Double,
        computed_at -> Timestamp,
    }
}

diesel::table! {
    file_summaries (file_id) {
        file_id -> Text,
        summary -> Text,
        generated_at -> Timestamp,
    }
}

diesel::table! {
    file_classifications (file_id) {
        file_id -> Text,
        labels -> Text,
        classified_at -> Timestamp,
    }
}

diesel::joinable!(files -> folders (folder_id));
diesel::joinable!(shortcuts -> files (target_file_id));
diesel::joinable!(file_versions -> files (file_id));

diesel::allow_tables_to_appear_in_same_query!(
    files,
    user_quotas,
    folders,
    shortcuts,
    file_versions,
    permissions,
    share_links,
    access_requests,
    irm_policies,
    workspace_settings,
    doc_suggestions,
    comments,
    comment_replies,
    notifications,
    file_activity_log,
    file_content_index,
    file_access_scores,
    file_summaries,
    file_classifications,
);
