-- Test-only, for `POST /e2e/reset` (see `src/handlers/e2e_reset.rs`): the
-- users behind the emails in the JSON array ?1.
SELECT user_id
FROM password_credentials
WHERE email IN (SELECT value FROM json_each(?1));
