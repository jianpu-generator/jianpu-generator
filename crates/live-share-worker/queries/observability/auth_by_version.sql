-- Do refusals cluster right after deploys? Auth events per worker version
-- (git SHA) and day, with when each version was first seen in the log
-- (roughly its deploy time). A jump in `sessions_rejected` without sign-outs
-- or password changes to explain it points at the deploy.
SELECT
    worker_version,
    DATE(at / 1000, 'unixepoch') AS day,
    DATETIME(MIN(at) / 1000, 'unixepoch') AS first_seen,
    SUM(kind IN ('signed_in', 'signed_up')) AS sessions_started,
    SUM(kind = 'sign_in_failed') AS sign_ins_failed,
    SUM(kind = 'locked_out') AS lockouts,
    SUM(kind = 'session_rejected') AS sessions_rejected,
    SUM(kind = 'signed_out') AS sign_outs,
    SUM(kind = 'password_changed') AS password_changes
FROM app_log
WHERE flow = 'auth'
GROUP BY worker_version, day
ORDER BY day DESC, first_seen DESC;
