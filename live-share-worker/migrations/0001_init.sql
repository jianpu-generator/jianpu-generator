-- Initial schema for Synced Share (Rust + D1 migration).
--
-- Provider-agnostic identity model: ownership points at an internal `users`
-- row, not a raw GitHub id, so a future non-GitHub identity provider can
-- link into the same `users` table without a schema change. See
-- TODO-synced-share-rust-d1-migration.md §1 for the design discussion.
--
-- This is a brand-new schema with no prior data to migrate.

CREATE TABLE users (
    id TEXT PRIMARY KEY,              -- unguessable random id, same style
                                       -- as share_id -- not sequential
    created_at INTEGER NOT NULL
);

CREATE TABLE user_identities (
    provider TEXT NOT NULL,           -- 'github' today; future: others
    provider_user_id TEXT NOT NULL,   -- TEXT: not every provider's id is numeric
    user_id TEXT NOT NULL REFERENCES users(id),
    login TEXT,                       -- best-effort cached display name
    linked_at INTEGER NOT NULL,
    PRIMARY KEY (provider, provider_user_id)
);

-- Reverse lookup: "this user's linked identities".
CREATE INDEX idx_user_identities_user_id ON user_identities(user_id);

CREATE TABLE oauth_sessions (
    token_hash TEXT PRIMARY KEY,      -- hash of the *Synced Share sign-in*
                                       -- token specifically (never the
                                       -- storage-backend token -- see §0)
    provider TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    verified_at INTEGER NOT NULL      -- ~1hr TTL checked in app code
);

CREATE TABLE docs (
    share_id TEXT PRIMARY KEY,
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    filename TEXT NOT NULL,
    content TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 0,
    ended INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Open items carried over, still to decide during implementation (not
-- resolved by this migration): whether `revision` acts as an
-- optimistic-concurrency guard on writes, and whether `owner_user_id`
-- needs an index for a possible future "my shares" list.
