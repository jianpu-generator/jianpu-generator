-- Who is minting many tokens? Tokens issued per GitHub user and browser
-- (`User-Agent`) over the last 7 days. More than 10 for one user means
-- GitHub has been silently revoking that user's oldest tokens.
SELECT
    user_agent,
    JSON_EXTRACT(detail, '$.provider_user_id') AS provider_user_id,
    JSON_EXTRACT(detail, '$.login') AS login,
    COUNT(*) AS tokens_issued,
    DATETIME(MAX(at) / 1000, 'unixepoch') AS last_issued_at
FROM app_log
WHERE
    flow = 'auth' AND kind = 'token_issued'
    AND at > (UNIXEPOCH() - 7 * 24 * 60 * 60) * 1000
GROUP BY provider_user_id, user_agent
ORDER BY tokens_issued DESC;
