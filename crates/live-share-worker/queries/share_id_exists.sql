-- Existence check used by the collision-handling step of server-side
-- share_id generation (see `src/share_id.rs`).
SELECT 1 AS present FROM docs WHERE share_id = ?1;
