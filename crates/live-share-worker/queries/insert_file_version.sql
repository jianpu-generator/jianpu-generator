-- Records one version: the reverse patch from the text at ?3 (the new head)
-- back to the text at ?2 (the previous head), saved at ?5.
INSERT INTO file_versions (file_id, revision, newer_revision, reverse_patch, saved_at)
VALUES (?1, ?2, ?3, ?4, ?5);
