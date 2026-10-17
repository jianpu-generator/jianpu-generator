-- get_folder_share_status
-- The owner's view of whether folder `?2` is shared; no row means this
-- caller never shared it.
SELECT share_id, ended_at
FROM shares
WHERE owner_user_id = ?1
  AND folder_prefix = ?2;
