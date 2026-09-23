-- Stop sharing a file. Owner-gated through `files.owner_user_id`, so zero
-- rows affected means "no share for a file this caller owns" (404). The
-- row is kept so a later start reproduces the same link.
UPDATE shares
SET ended_at = ?1
WHERE file_id = ?2
  AND file_id IN (SELECT id FROM files WHERE owner_user_id = ?3);
