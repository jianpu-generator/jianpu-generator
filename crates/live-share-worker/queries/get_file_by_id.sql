-- Scoped by id + owner_user_id, matching this crate's identity-gating
-- convention -- no route ever looks up a file by id alone. Used both for
-- normal reads and as the post-conflict lookup after a zero-rows-affected
-- content write (see `files::classify_content_write`).
SELECT id,
       owner_user_id,
       name,
       content,
       revision,
       trashed_at,
       created_at,
       updated_at
FROM files
WHERE id = ?1
  AND owner_user_id = ?2;
