-- Change password: the signed-in user's own credentials.
SELECT user_id,
       email,
       password_hash
FROM password_credentials
WHERE user_id = ?1;
