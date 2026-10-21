-- Sign-in: the account behind an email (stored lowercased).
SELECT user_id,
       email,
       password_hash
FROM password_credentials
WHERE email = ?1;
