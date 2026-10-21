-- Test-only hard delete for `POST /e2e/reset`: every session of the users
-- in the JSON array ?1.
DELETE FROM sessions
WHERE user_id IN (SELECT value FROM json_each(?1));
