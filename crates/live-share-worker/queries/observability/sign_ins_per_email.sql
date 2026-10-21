-- Who is signing in a lot, or being guessed at? Sessions started and
-- sign-ins refused per email and browser (`User-Agent`) over the last 7
-- days. Many `failed` with `unknown_email` across many emails from one
-- browser looks like someone trying addresses.
SELECT
    subject,
    user_agent,
    SUM(kind IN ('signed_in', 'signed_up')) AS sessions_started,
    SUM(kind = 'sign_in_failed') AS failed,
    SUM(
        kind = 'sign_in_failed'
        AND JSON_EXTRACT(detail, '$.reason') = 'unknown_email'
    ) AS unknown_email,
    SUM(kind = 'locked_out') AS lockouts,
    DATETIME(MAX(at) / 1000, 'unixepoch') AS last_seen_at
FROM app_log
WHERE
    flow = 'auth'
    AND kind IN ('signed_in', 'signed_up', 'sign_in_failed', 'locked_out')
    AND at > (UNIXEPOCH() - 7 * 24 * 60 * 60) * 1000
GROUP BY subject, user_agent
ORDER BY failed DESC, sessions_started DESC;
