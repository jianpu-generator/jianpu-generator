-- Atomic restore. `name` is supplied by the caller (already recomputed
-- client-side to be unique against the caller's current in-memory state,
-- per fileStore.ts's uniqueName/reservedNames) -- this can still fail the
-- idx_files_owner_name unique index on a genuine collision race, mapped to
-- a 409 name_taken response the same way insert_file/rename_file are (see
-- handlers.rs's is_unique_constraint_violation). Guarded on trashed_at IS
-- NOT NULL so restoring an already-active file is a no-op 404, not a
-- silent rename.
UPDATE files
SET trashed_at = NULL,
    name = ?1,
    updated_at = ?2
WHERE id = ?3
  AND owner_user_id = ?4
  AND trashed_at IS NOT NULL;
