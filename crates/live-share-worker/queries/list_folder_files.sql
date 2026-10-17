-- list_folder_files
-- Every active file of owner ?1 under folder ?2 (placeholders included), in
-- path order. Same prefix match as `get_folder_files_stats.sql`.
SELECT name,
       content,
       revision
FROM files
WHERE owner_user_id = ?1
  AND trashed_at IS NULL
  AND substr(name, 1, length(?2) + 1) = ?2 || '/'
ORDER BY name;
