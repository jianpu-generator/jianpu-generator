-- Hashed-token cache lookup (TODO §0/§6, task 7): looks up a previous
-- verification for this exact token hash. Freshness (the ~1hr TTL) is
-- checked in app code against `verified_at`, not here -- see
-- `src/verification.rs`'s `session_is_fresh`.
SELECT provider, provider_user_id, verified_at
FROM oauth_sessions
WHERE token_hash = ?1;
