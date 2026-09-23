-- Start (or resume) sharing a file. `?1` is a freshly generated share_id,
-- only used when the file has never been shared; on conflict with the
-- `UNIQUE(file_id)` constraint the existing row keeps its share_id and is
-- just marked live again, so re-sharing a file always reproduces the same
-- link -- idempotent and race-safe without a separate lookup. Callers must
-- check the caller owns `?2` first (see `crate::handlers::shares`).
INSERT INTO shares (share_id, file_id, ended_at, created_at)
VALUES (?1, ?2, NULL, ?3)
ON CONFLICT (file_id) DO UPDATE SET ended_at = NULL
RETURNING share_id;
