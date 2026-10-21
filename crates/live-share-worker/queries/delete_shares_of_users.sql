-- Test-only hard delete for `POST /e2e/reset` (see
-- `src/handlers/e2e_reset.rs`): every share (file or folder) owned by the
-- users in the JSON array ?1. Runs before `delete_files_of_users.sql`,
-- since `shares.file_id` references `files(id)`.
DELETE FROM shares
WHERE owner_user_id IN (SELECT value FROM json_each(?1));
