-- Atomic delete (move to the bin). Guarded on trashed_at IS NULL so an
-- already-trashed file's trashed_at timestamp isn't silently bumped by a
-- second delete call -- zero rows affected there reads as "gone" (404),
-- which is accurate: from this route's perspective there's no active file
-- with this id to delete.
UPDATE files
SET trashed_at = ?1,
    updated_at = ?1
WHERE id = ?2
  AND owner_user_id = ?3
  AND trashed_at IS NULL;
