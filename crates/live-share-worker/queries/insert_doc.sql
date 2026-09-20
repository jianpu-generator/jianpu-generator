-- Creates a new share row at `POST /shares` time. `ended` is bound as an
-- integer (0/1) -- D1/SQLite has no native boolean column type.
-- `external_file_id` is `NULL` for a local-only file -- see
-- `migrations/0002_docs_external_file_id.sql`.
INSERT INTO docs (
    share_id,
    owner_user_id,
    filename,
    content,
    revision,
    ended,
    created_at,
    updated_at,
    external_file_id
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);
