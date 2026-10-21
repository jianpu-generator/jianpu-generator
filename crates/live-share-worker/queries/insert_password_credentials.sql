-- A new account's credentials, inserted in the same batch as its `users`
-- row (`insert_user.sql`). Fails on `password_credentials.email`'s UNIQUE
-- constraint when the email already has an account.
INSERT INTO password_credentials (
    user_id, email, password_hash, created_at, updated_at
)
VALUES (?1, ?2, ?3, ?4, ?4);
