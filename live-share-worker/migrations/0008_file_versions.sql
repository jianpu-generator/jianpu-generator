-- Version history of cloud files (see `crates/live-share-worker/src/file_history.rs`).
--
-- `files.content` stays the full current text. Each accepted content save
-- that changes the text stores one row here: a reverse unified diff that
-- turns the text at `newer_revision` back into the text at `revision`. An
-- old version is rebuilt by applying patches backward from the head,
-- following the `newer_revision` links. Only the newest
-- `file_history::MAX_VERSIONS_PER_FILE` rows of a file are kept.
--
-- Keyed by `files.id`, so a rename never orphans history. Trashed files keep
-- theirs; deleting a `files` row (a purge) cascades.

CREATE TABLE file_versions (
    file_id        TEXT    NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    revision       INTEGER NOT NULL,
    newer_revision INTEGER NOT NULL,
    reverse_patch  TEXT    NOT NULL,
    saved_at       INTEGER NOT NULL,   -- when the text at `revision` was saved
    PRIMARY KEY (file_id, revision)
);

CREATE UNIQUE INDEX idx_file_versions_newer ON file_versions(file_id, newer_revision);
