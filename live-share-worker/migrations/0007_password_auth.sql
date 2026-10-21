-- Email + password accounts replace GitHub sign-in (see
-- `crates/live-share-worker/src/auth/`). A user's `users` row is unchanged;
-- `password_credentials` is now the one way to sign in to it.
--
-- `user_identities` (the GitHub link) is kept, read by nothing at runtime,
-- as the record of which GitHub account owned which user: the admin query
-- that hands a former GitHub user their old files
-- (`crates/live-share-worker/queries/admin/link_credentials_to_user.sql`)
-- finds the old user through it. A later migration can drop it once nobody
-- is left to move over.

-- One per user. `email` is stored trimmed and lowercased. `password_hash`
-- is self-describing: `pbkdf2-sha256$<iterations>$<salt b64>$<hash b64>`.
CREATE TABLE password_credentials (
    user_id TEXT PRIMARY KEY REFERENCES users(id),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- A signed-in browser. `token_hash` is the SHA-256 hex digest of the random
-- session token the browser holds; the token itself is never stored.
-- `expires_at` slides forward as the session is used.
CREATE TABLE sessions (
    token_hash TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);

-- Failed sign-ins per email, whether or not the email has an account, so a
-- lockout never tells anyone whether it does. `locked_until` is set once
-- `failed_count` reaches the limit; `lockout_level` doubles the pause each
-- time the limit is reached again. A successful sign-in deletes the row.
CREATE TABLE sign_in_lockouts (
    email TEXT PRIMARY KEY,
    failed_count INTEGER NOT NULL,
    locked_until INTEGER,
    lockout_level INTEGER NOT NULL DEFAULT 0
);

-- The GitHub token verification cache; nothing reads it any more.
DROP TABLE oauth_sessions;
