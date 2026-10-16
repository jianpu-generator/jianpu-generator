-- get_share_status_by_file
-- The owner's view of whether a file is shared, for any device (replaces
-- the old per-browser localStorage flag). Owner-gated the same way as
-- `end_share.sql`; no row means the file has never been shared by this
-- caller.
SELECT share_id, ended_at
FROM shares
WHERE file_id = ?1
  AND owner_user_id = ?2;
