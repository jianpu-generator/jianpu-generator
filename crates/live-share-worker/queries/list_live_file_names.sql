-- list_live_file_names
-- The id and name of every active (not trashed) file of owner ?1: the final
-- state a write is checked against so no path is both a file and a folder.
SELECT id,
       name
FROM files
WHERE owner_user_id = ?1
  AND trashed_at IS NULL;
