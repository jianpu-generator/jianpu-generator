-- Idempotent-create lookup: does this owner already have a share for this
-- external file? See `migrations/0002_docs_external_file_id.sql`'s partial
-- unique index and `crate::handlers::create_share`.
SELECT share_id FROM docs
WHERE owner_user_id = ?1 AND external_file_id = ?2;
