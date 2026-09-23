-- The owner's view of whether a file is shared, for any device (replaces
-- the old per-browser localStorage flag). Owner-gated the same way as
-- `end_share.sql`; no row means the file has never been shared by this
-- caller.
SELECT s.share_id, s.ended_at
FROM shares s
JOIN files f ON f.id = s.file_id
WHERE s.file_id = ?1
  AND f.owner_user_id = ?2;
