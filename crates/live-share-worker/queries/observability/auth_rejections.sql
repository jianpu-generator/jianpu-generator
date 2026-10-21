-- Why was a user signed out, or refused at sign-in? The last 50 refused
-- auth events, newest first. `sign_in_failed` gives the email and `reason`
-- (`wrong_password`, `unknown_email`, `locked`); `locked_out` says until
-- when. `session_rejected` means a session token was refused (signed out,
-- expired, or ended by a password change): `started_by` is the sign-in or
-- sign-up that issued it, and `password_changed_at` the account's latest
-- password change after it, if any. No row around the time a user was
-- signed out => the worker never answered 401, so the browser dropped the
-- token itself.
SELECT
    r.kind,
    r.subject,
    JSON_EXTRACT(r.detail, '$.reason') AS reason,
    JSON_EXTRACT(r.detail, '$.failed_count') AS failed_count,
    DATETIME(JSON_EXTRACT(r.detail, '$.locked_until') / 1000, 'unixepoch')
        AS locked_until,
    s.subject AS started_by,
    DATETIME(s.at / 1000, 'unixepoch') AS started_at,
    (
        SELECT DATETIME(MAX(p.at) / 1000, 'unixepoch') FROM app_log AS p
        WHERE
            p.flow = 'auth' AND p.kind = 'password_changed'
            AND p.subject = s.subject AND p.at > s.at
    ) AS password_changed_at,
    r.worker_version,
    r.origin,
    r.user_agent,
    DATETIME(r.at / 1000, 'unixepoch') AS rejected_at
FROM app_log AS r
LEFT JOIN app_log AS s
    ON
        r.kind = 'session_rejected'
        AND s.flow = 'auth' AND s.kind IN ('signed_in', 'signed_up')
        AND JSON_EXTRACT(s.detail, '$.token_tag')
        = JSON_EXTRACT(r.detail, '$.token_tag')
WHERE r.flow = 'auth' AND r.outcome = 'rejected'
ORDER BY r.at DESC
LIMIT 50;
