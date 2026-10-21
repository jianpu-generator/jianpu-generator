-- Sign-out: ends one session.
DELETE FROM sessions
WHERE token_hash = ?1;
