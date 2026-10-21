-- Test-only hard delete for `POST /e2e/reset`: the failed sign-ins of the
-- emails in the JSON array ?1, account or not.
DELETE FROM sign_in_lockouts
WHERE email IN (SELECT value FROM json_each(?1));
