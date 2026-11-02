-- Every stored version of a file, for the caller ?2 only (a file that is not
-- theirs has no rows here).
SELECT v.revision,
       v.newer_revision,
       v.reverse_patch,
       v.saved_at
FROM file_versions v
JOIN files f ON f.id = v.file_id
WHERE v.file_id = ?1
  AND f.owner_user_id = ?2
ORDER BY v.revision DESC;
