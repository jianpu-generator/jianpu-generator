-- get_folder_share_header
-- The anonymous viewer read for `GET /shares/:share_id` when the id is a
-- folder share: the share's own state, which folder it names and whose
-- files to read (`owner_user_id` stays inside the worker).
SELECT s.ended_at,
       s.owner_user_id,
       s.folder_prefix
FROM shares s
WHERE s.share_id = ?1
  AND s.folder_prefix IS NOT NULL;
