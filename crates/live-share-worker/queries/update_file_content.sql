-- The atomic CAS guard for a content save: only applies when the caller
-- owns the file, it's still active (not trashed), and it's still at the
-- revision the caller last saw. Zero rows affected means one of those
-- didn't hold -- `files::classify_content_write` (fed by `get_file_by_id`)
-- decides whether that's a conflict or a not-found.
UPDATE files
SET content = ?1,
    revision = revision + 1,
    updated_at = ?2
WHERE id = ?3
  AND owner_user_id = ?4
  AND revision = ?5
  AND trashed_at IS NULL;
