-- A new signed-in session. Only the token's hash is stored.
INSERT INTO sessions (token_hash, user_id, created_at, expires_at)
VALUES (?1, ?2, ?3, ?4);
