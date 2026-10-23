-- Keeps only the newest ?2 versions of file ?1.
DELETE FROM file_versions
WHERE file_id = ?1
  AND revision NOT IN (
    SELECT revision
    FROM file_versions
    WHERE file_id = ?1
    ORDER BY revision DESC
    LIMIT ?2
  );
