-- end_folder_share
-- Stop sharing folder `?3` of owner `?2`. Zero rows affected means this
-- caller never shared that folder (404). The row is kept so a later start
-- reproduces the same link.
UPDATE shares
SET ended_at = ?1
WHERE owner_user_id = ?2
  AND folder_prefix = ?3;
