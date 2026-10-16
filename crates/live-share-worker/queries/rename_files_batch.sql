-- Renames every file of owner ?3 listed in the JSON array ?1 (objects with
-- `id` and `name`) in one statement, so it is all-or-nothing: a unique-index
-- violation (idx_files_owner_name) on any row rolls the whole statement
-- back. No revision gate, like rename_file.sql.
UPDATE files
SET name = (
        SELECT json_extract(value, '$.name')
        FROM json_each(?1)
        WHERE json_extract(value, '$.id') = files.id
    ),
    updated_at = ?2
WHERE owner_user_id = ?3
  AND id IN (SELECT json_extract(value, '$.id') FROM json_each(?1));
