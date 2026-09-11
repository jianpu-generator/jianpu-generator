-- create-on-first-sight: the `users` row half of a new identity, inserted
-- before `insert_user_identity.sql` in the same request.
INSERT INTO users (id, created_at) VALUES (?1, ?2);
