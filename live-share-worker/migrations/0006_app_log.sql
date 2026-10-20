-- Append-only event log for debugging production from data rather than
-- guesses (see `crates/live-share-worker/src/app_log.rs`). One generic table
-- for every flow: `flow` + `kind` name the event, `detail` is its JSON
-- payload whose shape is fixed per (flow, kind) by the Rust `LogEvent` enum.
-- The DB only sees text; the type safety lives in Rust.
--
-- Rows are events, never per-request successes, so volume stays at tens of
-- rows per day. Rows older than 30 days are pruned by the worker's daily
-- scheduled handler (`queries/prune_app_log.sql`). Never stores a token:
-- `subject` identifies one by its token tag (`token:<first 12 hex chars of
-- sha256(token)>`, e.g. `token:3fa9c1d20b7e`).
--
-- `outcome` is one of 'ok' | 'rejected' | 'unavailable' | 'error'. `detail`
-- is JSON whose shape is fixed per (flow, kind) by the Rust enums.
--
-- Agents query it with the saved queries in
-- `crates/live-share-worker/queries/observability/`.

CREATE TABLE app_log (
    id             INTEGER PRIMARY KEY,
    at             INTEGER NOT NULL,  -- ms since epoch
    flow           TEXT NOT NULL,     -- 'auth' (later: 'files', 'share', ...)
    kind           TEXT NOT NULL,     -- unique within its flow, snake_case
    outcome        TEXT NOT NULL,     -- see above
    user_id        TEXT,              -- internal users.id, when resolved
    subject        TEXT,              -- typed tag of what the row is about
    worker_version TEXT,              -- git SHA the worker was built from
    origin         TEXT,              -- request Origin header, when captured
    user_agent     TEXT,              -- request User-Agent, when captured
    detail         TEXT               -- JSON, see above
);

CREATE INDEX idx_app_log_at ON app_log(at);
CREATE INDEX idx_app_log_flow_kind_at ON app_log(flow, kind, at);
CREATE INDEX idx_app_log_subject ON app_log(subject);
