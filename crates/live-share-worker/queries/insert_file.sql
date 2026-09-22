-- Creates a new file row (new file / duplicate / import). `id` is
-- client-generated (fileStore.ts's generateFileId()) -- safe because every
-- write is additionally gated on owner_user_id everywhere else. Can fail
-- the idx_files_owner_name unique index on a genuine name-collision race --
-- see `handlers.rs`'s `is_unique_constraint_violation` handling.
INSERT INTO files (
    id,
    owner_user_id,
    name,
    content,
    revision,
    trashed_at,
    created_at,
    updated_at
)
VALUES (?1, ?2, ?3, ?4, 0, NULL, ?5, ?6);
