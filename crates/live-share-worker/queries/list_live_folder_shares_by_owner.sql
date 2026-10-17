-- list_live_folder_shares_by_owner
-- Part of `POST /shares/overview`: the owner's live folder links.
SELECT share_id,
       folder_prefix
FROM shares
WHERE owner_user_id = ?1
  AND folder_prefix IS NOT NULL
  AND ended_at IS NULL
ORDER BY folder_prefix;
