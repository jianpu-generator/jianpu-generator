-- Cloud storage backend: files table, replacing the GitHub Contents API
-- storage backend with D1, unified under the same account identity Synced
-- Share already has (see `crate::files`'s module doc comment).
--
-- A single table, not a separate trash table: `trashed_at` (nullable
-- timestamp) models delete/restore as a single atomic `UPDATE`, keeps
-- `revision` continuous across a delete/restore cycle, and avoids a second
-- id space to reconcile.

CREATE TABLE files (
    id TEXT PRIMARY KEY,               -- client-generated (fileStore.ts's
                                        -- generateFileId()); safe because
                                        -- every write is additionally gated
                                        -- on owner_user_id.
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    content TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 0,   -- content-write CAS guard only --
                                            -- rename/delete/restore are
                                            -- unconditional atomic UPDATEs,
                                            -- not gated on this.
    trashed_at INTEGER,                    -- NULL = active, else in the bin
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_files_owner_user_id ON files(owner_user_id);

-- Enforced, not just client-trusted: `web/src/fileStore.ts`'s pure
-- `reservedNames` already folds active *and* binned names into one reserved
-- set before computing a unique name client-side (so the client itself
-- never knowingly asks for a colliding name, trashed or not) -- this index
-- makes that invariant a real guarantee against a genuine race (e.g. two
-- tabs/devices both creating "untitled.jianpu" around the same time), not
-- just an assumption. Unconditional (not partial on trashed_at), matching
-- that same client-side invariant, which treats the bin as reserved too.
CREATE UNIQUE INDEX idx_files_owner_name ON files(owner_user_id, name);
