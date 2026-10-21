-- Test-only hard delete for `POST /e2e/reset`: every file, active or
-- binned, owned by the users in the JSON array ?1.
DELETE FROM files
WHERE owner_user_id IN (SELECT value FROM json_each(?1));
