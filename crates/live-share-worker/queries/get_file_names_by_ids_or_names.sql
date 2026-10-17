-- The id and name of every file of owner ?1 that is one of the ids in the
-- JSON array ?2 or holds one of the names in the JSON array ?3: what a batch
-- rename needs to tell "all there", "already applied" and "name taken" apart
-- before writing. Trashed files count: they keep their name.
SELECT id,
       name
FROM files
WHERE owner_user_id = ?1
  AND (
    id IN (SELECT value FROM json_each(?2))
    OR name IN (SELECT value FROM json_each(?3))
  );
