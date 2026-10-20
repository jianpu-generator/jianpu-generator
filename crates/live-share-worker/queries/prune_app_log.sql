-- Deletes up to ?2 of the oldest `app_log` rows older than ?1 (ms since
-- epoch). Run in a loop by the daily scheduled handler until a batch comes
-- back short, so one prune never becomes a single huge D1 write.
DELETE FROM app_log
WHERE id IN (SELECT id FROM app_log WHERE at < ?1 ORDER BY at LIMIT ?2);
