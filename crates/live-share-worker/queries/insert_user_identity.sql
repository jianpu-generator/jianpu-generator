-- create-on-first-sight: the `user_identities` row half of a new identity.
-- `login` (?4) is nullable -- populated by `GithubIdentityProvider` from
-- `GET /user`'s `login` field (best-effort display name, per the schema
-- comment); a future non-GitHub provider that can't offer one binds NULL.
INSERT INTO user_identities (
    provider, provider_user_id, user_id, login, linked_at
)
VALUES (?1, ?2, ?3, ?4, ?5);
