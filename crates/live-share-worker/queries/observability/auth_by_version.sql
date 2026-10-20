-- Do rejections cluster right after deploys? Cache-miss verification
-- outcomes per worker version (git SHA) and day, with when each version was
-- first seen in the log (roughly its deploy time).
SELECT
    worker_version,
    DATE(at / 1000, 'unixepoch') AS day,
    DATETIME(MIN(at) / 1000, 'unixepoch') AS first_seen,
    SUM(outcome = 'rejected') AS rejected,
    SUM(outcome = 'unavailable') AS unavailable,
    COUNT(*) AS verifications
FROM app_log
WHERE
    flow = 'auth'
    AND kind IN ('token_verified', 'token_rejected', 'verification_unavailable')
GROUP BY worker_version, day
ORDER BY day DESC, first_seen DESC;
