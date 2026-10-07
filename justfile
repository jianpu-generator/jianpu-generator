# Local dev runner for testing "Sign in with GitHub" against the real GitHub
# OAuth App. Starts the worker and the Vite dev server together, streaming
# plain [worker]/[web]-prefixed logs to stdout and ./dev.log (no TUI, so it is
# safe to run in the background and tail the log). Ctrl-C / kill stops both.
#
# `cf` and `wrangler` are dev dependencies of crates/live-share-worker; run
# `pnpm install` there once first. The web side talks to the local worker via
# web/.env.local's VITE_SYNCED_SHARE_HOST=localhost:8787.
dev:
    #!/usr/bin/env bash
    set -uo pipefail
    trap 'kill 0' EXIT INT TERM
    : > dev.log
    just dev-worker 2>&1 | sed -u 's/^/[worker] /' | tee -a dev.log &
    just dev-web 2>&1 | sed -u 's/^/[web] /' | tee -a dev.log &
    wait

# `cf dev` for crates/live-share-worker, real GitHub OAuth creds from its
# gitignored .env.local (fill in SYNCED_SHARE_GITHUB_CLIENT_SECRET first). cf
# has no `--var`, so the secret goes in via `WORKER_VAR_OVERRIDES` (see
# `cloudflare.config.ts`). Deliberately plain HTTP (no `--local-protocol
# https`): cf dev's self-signed cert is silently rejected by browsers (fetch()
# surfaces an opaque "Failed to fetch"), and nothing locally needs HTTPS (see
# `web/src/syncedShare/workerUrl.ts`).
[private]
dev-worker:
    #!/usr/bin/env bash
    set -euo pipefail
    cd crates/live-share-worker
    set -a && . ./.env.local && set +a
    export WORKER_VAR_OVERRIDES="{\"SYNCED_SHARE_GITHUB_CLIENT_SECRET\":\"$SYNCED_SHARE_GITHUB_CLIENT_SECRET\"}"
    pnpm exec cf d1 migrations apply "$(jq -r .d1DatabaseId deploy.json)" --local --dir ../../live-share-worker/migrations
    exec pnpm exec cf dev --port 8787

[private]
dev-web:
    cd web && pnpm dev
