-- Backfill thumbnail jobs for all existing files that have no cover thumbnail yet.
--
-- Supported MIME types mirror the worker's generate_thumbnail_for_type() dispatcher:
--   image/*  · application/pdf  · docx/pptx  · text/csv
--   application/x-neutrino-{doc,sheet,slide}
--
-- Jobs are inserted with status 'R' (ready) so the drive background task will
-- dispatch them to any registered worker.  Each file gets at most one new job:
-- files that already have a cover_thumbnail are skipped.
--
-- Run against the SQLite database:
--   sqlite3 neutrino.db < backfill_thumbnail_jobs.sql

INSERT INTO worker_jobs (id, job_type, payload, status, timeout_secs, created_at, updated_at)
SELECT
    lower(hex(randomblob(4)))  || '-'
    || lower(hex(randomblob(2))) || '-'
    || lower(hex(randomblob(2))) || '-'
    || lower(hex(randomblob(2))) || '-'
    || lower(hex(randomblob(6)))        AS id,
    'thumbnail'                         AS job_type,
    json_object('fileId', f.id)         AS payload,
    'R'                                 AS status,
    60                                  AS timeout_secs,
    CURRENT_TIMESTAMP                   AS created_at,
    CURRENT_TIMESTAMP                   AS updated_at
FROM files f
WHERE
    -- Skip files that already have a thumbnail
    (f.cover_thumbnail IS NULL OR f.cover_thumbnail = '')
    -- Skip soft-deleted files
    AND f.deleted_at IS NULL
    -- Only supported MIME types
    AND (
        f.mime_type LIKE 'image/%'
        OR f.mime_type = 'application/pdf'
        OR f.mime_type = 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'
        OR f.mime_type = 'application/msword'
        OR f.mime_type = 'application/vnd.openxmlformats-officedocument.presentationml.presentation'
        OR f.mime_type = 'application/vnd.ms-powerpoint'
        OR f.mime_type = 'text/csv'
        OR f.mime_type = 'application/csv'
        OR f.mime_type = 'text/comma-separated-values'
        OR f.mime_type = 'application/x-neutrino-doc'
        OR f.mime_type = 'application/x-neutrino-sheet'
        OR f.mime_type = 'application/x-neutrino-slide'
    )
    -- Don't create a duplicate if a ready/in-progress job already exists
    AND NOT EXISTS (
        SELECT 1
        FROM worker_jobs wj
        WHERE wj.job_type = 'thumbnail'
          AND json_extract(wj.payload, '$.fileId') = f.id
          AND wj.status IN ('R', 'I')
    );

SELECT 'Inserted ' || changes() || ' thumbnail job(s).' AS result;
