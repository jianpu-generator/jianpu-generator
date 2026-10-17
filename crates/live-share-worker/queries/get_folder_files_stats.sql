-- get_folder_files_stats
-- How many active files of owner ?1 lie under folder ?2 (placeholders
-- included) and how many bytes of content they hold, so a folder share can
-- be refused (or a viewer read answered with an error) before the content is
-- loaded. The prefix is matched with substr, not LIKE, so `%` and `_` in a
-- folder name are literal, and `Hymns` never matches `Hymns 2/...`.
SELECT COUNT(*) AS file_count,
       COALESCE(SUM(LENGTH(CAST(content AS BLOB))), 0) AS content_bytes
FROM files
WHERE owner_user_id = ?1
  AND trashed_at IS NULL
  AND substr(name, 1, length(?2) + 1) = ?2 || '/';
