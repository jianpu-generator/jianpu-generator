-- rewrite_folder_share_prefix
-- Points owner ?2's folder share at ?1 instead of ?3. Run in the same D1 batch
-- (one transaction) as `rename_files_batch.sql`.
UPDATE shares
SET folder_prefix = ?1
WHERE owner_user_id = ?2
  AND folder_prefix = ?3;
