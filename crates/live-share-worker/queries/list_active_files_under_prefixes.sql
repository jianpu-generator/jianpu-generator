-- list_active_files_under_prefixes
-- The id and name of every active file of owner ?1 lying under one of the
-- folders in the JSON array ?2: what a batch rename needs to tell which
-- shared folders it moves as a whole.
SELECT f.id,
       f.name
FROM files f
WHERE f.owner_user_id = ?1
  AND f.trashed_at IS NULL
  AND EXISTS (
    SELECT 1
    FROM json_each(?2) prefix
    WHERE substr(f.name, 1, length(prefix.value) + 1) = prefix.value || '/'
  );
