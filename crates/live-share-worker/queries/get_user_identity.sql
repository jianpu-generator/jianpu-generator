-- Looks up the internal user_id for a (provider, provider_user_id) pair.
-- Returns no row the first time a given identity is ever seen -- callers
-- then create-on-first-sight (see insert_user.sql / insert_user_identity.sql).
SELECT user_id
FROM user_identities
WHERE provider = ?1
  AND provider_user_id = ?2;
