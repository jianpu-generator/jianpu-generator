-- Synced Share: a share is a pointer to a cloud `files` row, not a copy of
-- its content.
--
-- `docs` kept its own `filename`/`content`/`revision` copy, pushed by the
-- owner's browser separately from autosave. A stale-render race in that
-- push (see `web/src/hooks/useSyncedShareOwner.ts`'s history) could write
-- one file's content to another file's share, so a viewer of link X ended
-- up seeing score Y. Pointing at `files` instead makes autosave the only
-- write path: a viewer reads `files.content` directly, so a share can never
-- show anything but its own file.
--
-- The owner is `files.owner_user_id` (no separate column to drift), and
-- "is this file shared" now lives here instead of per-browser localStorage,
-- so every device sees the same live/stopped state.

CREATE TABLE shares (
    share_id   TEXT PRIMARY KEY,
    file_id    TEXT NOT NULL UNIQUE REFERENCES files(id),  -- one link per file
    ended_at   INTEGER,                                    -- NULL = live
    created_at INTEGER NOT NULL
);

-- Keep links whose external_file_id is really a files.id owned by the same
-- user. Pre-cloud rows (GitHub paths / NULL local shares) are dropped.
INSERT INTO shares (share_id, file_id, ended_at, created_at)
SELECT d.share_id, d.external_file_id,
       CASE WHEN d.ended != 0 THEN d.updated_at END AS ended_at, d.created_at
FROM docs d
JOIN files f ON f.id = d.external_file_id AND f.owner_user_id = d.owner_user_id;

DROP INDEX idx_docs_owner_external_file;
DROP TABLE docs;
