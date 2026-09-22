-- Atomic rename, no revision gate -- renaming has never had conflict
-- semantics (matching the old GitHub backend's actual behavior). Can fail
-- the idx_files_owner_name unique index on a genuine name-collision race --
-- see `handlers.rs`'s `is_unique_constraint_violation` handling.
UPDATE files
SET name = ?1,
    updated_at = ?2
WHERE id = ?3
  AND owner_user_id = ?4;
