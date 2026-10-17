-- get_folder_share_header
-- The anonymous viewer read for `GET /shares/:share_id` when the id is a
-- folder share: the share's own state, which folder it names and whose
-- files to read (`owner_user_id` stays inside the worker). `owner_login` as
-- in `get_share_view.sql`.
SELECT s.ended_at,
       s.owner_user_id,
       s.folder_prefix,
       (SELECT ui.login
        FROM user_identities ui
        WHERE ui.user_id = s.owner_user_id
        LIMIT 1) AS owner_login
FROM shares s
WHERE s.share_id = ?1
  AND s.folder_prefix IS NOT NULL;
