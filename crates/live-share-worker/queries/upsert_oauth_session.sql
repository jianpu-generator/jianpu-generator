-- Refreshes (or inserts) the cached verification for a token hash after a
-- successful GitHub verification (TODO §0/§6, task 7). `token_hash` is the
-- primary key, so a re-verification of the same token just bumps
-- `verified_at` (and updates `provider`/`provider_user_id`, though those
-- should never actually change for the same hash).
INSERT INTO oauth_sessions (token_hash, provider, provider_user_id, verified_at)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT (token_hash) DO UPDATE SET
    provider = excluded.provider,
    provider_user_id = excluded.provider_user_id,
    verified_at = excluded.verified_at;
