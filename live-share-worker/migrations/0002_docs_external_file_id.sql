-- Idempotent Synced Share creation, keyed by GitHub account + GitHub file.
--
-- `external_file_id` is the client-derived `owner/repo/scores/name` Contents
-- API path for a GitHub-backed file (see `web/src/hooks/useScoreSource.ts`),
-- `NULL` for a local-only file (there's no account-independent identity to
-- key those off -- see the feature's design doc). Nullable rather than a
-- separate table: it's a single optional attribute of a `docs` row, not a
-- relationship needing its own lifecycle.
ALTER TABLE docs ADD COLUMN external_file_id TEXT;

-- Idempotent create-share invariant: at most one share per (owner, external
-- file), so `POST /shares` for a file already shared by this owner can look
-- this up and hand back the existing `share_id` instead of minting a new
-- one -- see `crate::handlers::create_share`. Partial index (nullable
-- column) so local-only shares, which have no `external_file_id`, are left
-- unconstrained -- `WHERE external_file_id IS NOT NULL` matches how SQLite
-- treats `NULL` as never equal to `NULL` in a UNIQUE index, made explicit
-- here rather than relied upon implicitly.
--
-- Resolves half of `0001_init.sql`'s trailing "open items" note (an index
-- involving `owner_user_id`).
CREATE UNIQUE INDEX idx_docs_owner_external_file
    ON docs(owner_user_id, external_file_id)
    WHERE external_file_id IS NOT NULL;
