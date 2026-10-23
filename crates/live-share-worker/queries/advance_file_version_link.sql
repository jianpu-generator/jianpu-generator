-- A save that left the text unchanged: the newest version's patch applies to
-- the new head ?3 just as it did to the old head ?2.
UPDATE file_versions
SET newer_revision = ?3
WHERE file_id = ?1
  AND newer_revision = ?2;
