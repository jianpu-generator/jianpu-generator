-- Slides a session's expiry forward as it is used.
UPDATE sessions
SET expires_at = ?2
WHERE token_hash = ?1;
