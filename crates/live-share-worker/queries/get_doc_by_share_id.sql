-- Reads a share's full row (including `owner_user_id`) by its public
-- share_id. Callers must strip `owner_user_id` before this ever reaches a
-- client -- see `to_public_doc` in `src/doc.rs`.
SELECT share_id,
       owner_user_id,
       filename,
       content,
       revision,
       ended,
       created_at,
       updated_at
FROM docs
WHERE share_id = ?1;
