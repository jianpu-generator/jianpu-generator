-- Test-only hard delete for `POST /e2e/reset`: the users in the JSON array
-- ?1, once nothing references them any more.
DELETE FROM users
WHERE id IN (SELECT value FROM json_each(?1));
