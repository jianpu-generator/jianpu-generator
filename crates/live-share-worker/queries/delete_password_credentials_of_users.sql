-- Test-only hard delete for `POST /e2e/reset`: the credentials of the
-- users in the JSON array ?1.
DELETE FROM password_credentials
WHERE user_id IN (SELECT value FROM json_each(?1));
