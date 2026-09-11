-- Creates a new share row at `POST /shares` time. `ended` is bound as an
-- integer (0/1) -- D1/SQLite has no native boolean column type.
INSERT INTO docs (
    share_id,
    owner_user_id,
    filename,
    content,
    revision,
    ended,
    created_at,
    updated_at
)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);
