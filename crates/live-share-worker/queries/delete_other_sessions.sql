-- Change password: signs the user out everywhere except the session that
-- changed it.
DELETE FROM sessions
WHERE user_id = ?1
  AND token_hash != ?2;
