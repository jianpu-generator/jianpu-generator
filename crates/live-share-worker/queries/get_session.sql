-- Resolves a session token's hash to its user, while it hasn't expired
-- (?2 is now).
SELECT user_id,
       expires_at
FROM sessions
WHERE token_hash = ?1
  AND expires_at > ?2;
