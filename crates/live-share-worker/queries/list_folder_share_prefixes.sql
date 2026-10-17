-- list_folder_share_prefixes
-- Every folder the owner has a share row for, live or stopped: a folder
-- rename must keep a stopped link pointing at the renamed folder too.
SELECT folder_prefix
FROM shares
WHERE owner_user_id = ?1
  AND folder_prefix IS NOT NULL;
