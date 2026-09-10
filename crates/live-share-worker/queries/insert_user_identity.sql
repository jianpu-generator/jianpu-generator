-- create-on-first-sight: the `user_identities` row half of a new identity.
-- `login` (?4) is nullable -- no display name is cached yet for the stub
-- provider; the real GitHub provider (task 6/7) can populate it.
INSERT INTO user_identities (
    provider, provider_user_id, user_id, login, linked_at
)
VALUES (?1, ?2, ?3, ?4, ?5);
