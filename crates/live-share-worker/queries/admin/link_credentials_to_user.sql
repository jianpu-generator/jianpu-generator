-- Admin only: never loaded by the worker. The owner runs it by hand against
-- production, from `crates/live-share-worker`, after replacing the two
-- placeholders below:
--
--   pnpm exec wrangler d1 execute jianpu-live-share --remote \
--     --command="$(cat queries/admin/link_credentials_to_user.sql)"
--
-- Moves an email account's credentials onto an existing user, so whoever
-- signs in with that email gets that user's files and shares. A former
-- GitHub user signs up with their email (creating a new, empty user), then
-- this hands them their old user back. Forgot password works the same way
-- once the admin has deleted the old credentials and the person has signed
-- up again.
--
-- Placeholders (never commit a real email or login in this file):
--   'OLD_GITHUB_LOGIN'  the GitHub login whose user should be kept
--   'NEW_EMAIL'         the email the person just signed up with
--
-- It changes nothing unless the new user owns no files and no shares, so it
-- can never orphan anyone's data, and fails if the old user already has
-- credentials (`password_credentials.user_id` is the primary key). Safe to
-- run twice.

-- 1. Sign the new, empty user out (the browser signs in again afterwards).
DELETE FROM sessions
WHERE user_id IN (
    SELECT pc.user_id
    FROM password_credentials pc
    WHERE pc.email = lower(trim('NEW_EMAIL'))
      AND NOT EXISTS (SELECT 1 FROM files f WHERE f.owner_user_id = pc.user_id)
      AND NOT EXISTS (SELECT 1 FROM shares s WHERE s.owner_user_id = pc.user_id)
);

-- 2. Point the credentials at the old user.
UPDATE password_credentials
SET user_id = (
        SELECT ui.user_id
        FROM user_identities ui
        WHERE ui.provider = 'github'
          AND ui.login = 'OLD_GITHUB_LOGIN'
    ),
    updated_at = unixepoch() * 1000
WHERE email = lower(trim('NEW_EMAIL'))
  AND EXISTS (
      SELECT 1
      FROM user_identities ui
      WHERE ui.provider = 'github'
        AND ui.login = 'OLD_GITHUB_LOGIN'
  )
  AND NOT EXISTS (
      SELECT 1 FROM files f
      WHERE f.owner_user_id = password_credentials.user_id
  )
  AND NOT EXISTS (
      SELECT 1 FROM shares s
      WHERE s.owner_user_id = password_credentials.user_id
  );

-- 3. Delete the now-unreachable new user: a `users` row nothing points at
-- (no credentials, identity, session, file or share). Every user is created
-- together with its credentials or identity, so the only such row is the
-- one step 2 just left behind.
DELETE FROM users
WHERE id NOT IN (SELECT user_id FROM password_credentials)
  AND id NOT IN (SELECT user_id FROM user_identities)
  AND id NOT IN (SELECT user_id FROM sessions)
  AND id NOT IN (SELECT owner_user_id FROM files)
  AND id NOT IN (SELECT owner_user_id FROM shares);
