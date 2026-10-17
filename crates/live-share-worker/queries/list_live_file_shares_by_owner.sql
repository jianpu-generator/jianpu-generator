-- list_live_file_shares_by_owner
-- Part of `POST /shares/overview`: the owner's live file links, skipping
-- files in the bin (their link reads as ended).
SELECT s.share_id,
       s.file_id,
       f.name
FROM shares s
JOIN files f ON f.id = s.file_id
WHERE s.owner_user_id = ?1
  AND s.ended_at IS NULL
  AND f.trashed_at IS NULL
ORDER BY f.name;
