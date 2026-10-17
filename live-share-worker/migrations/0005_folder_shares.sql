-- Synced Share of a whole folder: a share is now either a file share (points
-- at one `files` row via `file_id`, as before) or a folder share (names a
-- folder by `folder_prefix`, e.g. `Hymns/Advent`, and covers every file whose
-- name starts with `<folder_prefix>/` at the moment it is read). Exactly one
-- of the two is set (the CHECK below). Files are matched by prefix, never by
-- id, so moving a file in or out of the folder changes what the link shows;
-- renaming the folder rewrites `folder_prefix` in the same transaction as the
-- file renames (see `queries/rename_files_batch.sql`'s caller).
--
-- A file share's owner used to be implied by `files.owner_user_id`; a folder
-- share has no file to ask, so `owner_user_id` is now a column of its own
-- (backfilled from the pointed-at file for the existing rows).
--
-- SQLite cannot relax a NOT NULL or add a CHECK in place, so the table is
-- rebuilt. Nothing references `shares`, so the swap needs no foreign-key
-- juggling. A folder has at most one link per owner (unique index below), so
-- stopping and starting again reuses the same `share_id`, like file shares.

CREATE TABLE shares_rebuilt (
    share_id      TEXT PRIMARY KEY,
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    file_id       TEXT UNIQUE REFERENCES files(id),  -- file share: one link per file
    folder_prefix TEXT,                              -- folder share: path without trailing slash
    ended_at      INTEGER,                           -- NULL = live
    created_at    INTEGER NOT NULL,
    CHECK ((file_id IS NULL) <> (folder_prefix IS NULL))
);

INSERT INTO shares_rebuilt (share_id, owner_user_id, file_id, folder_prefix, ended_at, created_at)
SELECT s.share_id, f.owner_user_id, s.file_id, NULL, s.ended_at, s.created_at
FROM shares s
JOIN files f ON f.id = s.file_id;

DROP TABLE shares;
ALTER TABLE shares_rebuilt RENAME TO shares;

CREATE UNIQUE INDEX idx_shares_owner_folder_prefix ON shares(owner_user_id, folder_prefix);
