-- Change password: stores the new hash.
UPDATE password_credentials
SET password_hash = ?2,
    updated_at = ?3
WHERE user_id = ?1;
