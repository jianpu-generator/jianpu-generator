-- upsert_share
-- Start (or resume) sharing a file. `?1` is a freshly generated share_id,
-- only used when the file has never been shared; on conflict with the
-- `UNIQUE(file_id)` constraint the existing row keeps its share_id and is
-- just marked live again, so re-sharing a file always reproduces the same
-- link -- idempotent and race-safe without a separate lookup. Callers must
-- check the caller (`?2`, the file's owner) owns `?3` first (see
-- `crate::handlers::shares`).
INSERT INTO shares (share_id, owner_user_id, file_id, folder_prefix, ended_at, created_at)
VALUES (?1, ?2, ?3, NULL, NULL, ?4)
ON CONFLICT (file_id) DO UPDATE SET ended_at = NULL
RETURNING share_id;
