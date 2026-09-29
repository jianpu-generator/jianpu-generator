-- Test-only hard delete for `POST /e2e/reset` (see
-- `src/handlers/e2e_reset.rs`): every file, active or binned, owned by the
-- users behind provider ?1's identities whose provider user id is in the
-- JSON array ?2.
DELETE FROM files
WHERE owner_user_id IN (
    SELECT user_id
    FROM user_identities
    WHERE provider = ?1
      AND provider_user_id IN (SELECT value FROM json_each(?2))
);
