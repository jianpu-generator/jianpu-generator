-- Persists an owner's write (update or stop) applied by `doc::apply_write`.
-- `owner_user_id` is intentionally never updated here -- ownership is fixed
-- at share creation (see TODO-synced-share-rust-d1-migration.md §0).
UPDATE docs
SET filename = ?2,
    content = ?3,
    revision = ?4,
    ended = ?5,
    updated_at = ?6
WHERE share_id = ?1;
