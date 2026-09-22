-- All of an owner's files -- active and trashed alike -- for `POST
-- /files/list`. The client partitions the result by `trashed_at` into
-- `userFiles`/`bin` itself (see `cloudBackend.ts`'s `load()`); this query
-- doesn't filter, so a single round trip covers both.
SELECT id,
       owner_user_id,
       name,
       content,
       revision,
       trashed_at,
       created_at,
       updated_at
FROM files
WHERE owner_user_id = ?1;
