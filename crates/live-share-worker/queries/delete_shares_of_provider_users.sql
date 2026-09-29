-- Test-only hard delete for `POST /e2e/reset` (see
-- `src/handlers/e2e_reset.rs`): every share of every file owned by the
-- users behind provider ?1's identities whose provider user id is in the
-- JSON array ?2. Runs before `delete_files_of_provider_users.sql`, since
-- `shares.file_id` references `files(id)`.
DELETE FROM shares
WHERE file_id IN (
    SELECT f.id
    FROM files f
    JOIN user_identities ui ON ui.user_id = f.owner_user_id
    WHERE ui.provider = ?1
      AND ui.provider_user_id IN (SELECT value FROM json_each(?2))
);
