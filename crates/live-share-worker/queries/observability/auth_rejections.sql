-- Why was a user signed out? For each token GitHub rejected (newest first):
-- when/where it was issued, and how many tokens were issued to the same
-- GitHub user after it. `issued_after >= 10` => GitHub's cap of 10 live
-- tokens per user + OAuth App + scope revoked it. No `token_rejected` row
-- around the sign-out time => the worker never answered 401, so the token
-- was dropped client-side. `github_says` is GitHub's verbatim reason.
-- `origin`/`user_agent` are the rejected request's own, falling back to the
-- sign-in's (rows from before 2026-10-20 only have them on `token_issued`).
SELECT
    r.subject,
    r.worker_version,
    COALESCE(r.origin, i.origin) AS origin,
    COALESCE(r.user_agent, i.user_agent) AS user_agent,
    DATETIME(r.at / 1000, 'unixepoch') AS rejected_at,
    DATETIME(i.at / 1000, 'unixepoch') AS issued_at,
    (
        SELECT COUNT(*) FROM app_log AS n
        WHERE
            n.flow = 'auth' AND n.kind = 'token_issued' AND n.at > i.at
            AND JSON_EXTRACT(n.detail, '$.provider_user_id')
            = JSON_EXTRACT(i.detail, '$.provider_user_id')
    ) AS issued_after,
    JSON_EXTRACT(r.detail, '$.reason') AS github_says
FROM app_log AS r
LEFT JOIN app_log AS i
    ON i.flow = 'auth' AND i.kind = 'token_issued' AND i.subject = r.subject
WHERE r.flow = 'auth' AND r.kind = 'token_rejected'
ORDER BY r.at DESC
LIMIT 20;
