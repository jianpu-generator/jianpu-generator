import type { PlaywrightTestConfig } from '@playwright/test'
import deployConfig from '../src/generated/live-share-worker/deployConfig.json' with {
  type: 'json',
}
import e2eWorkerConfig from '../src/generated/live-share-worker/e2eWorkerConfig.json' with {
  type: 'json',
}
import { E2E_GITHUB_CLIENT_ID } from './e2eGithubClientId.ts'
import { KNOWN_GITHUB_USER_IDS } from './mockGithubIdentity.ts'

// The servers the e2e suite needs. Shared by `playwright.config.ts` (which
// lets Playwright start/reuse them) and `scripts/resolve-e2e-flakes.ts`
// (which starts them once and keeps them up across all of its passes).

const MOCK_GITHUB_ORIGIN = 'http://localhost:8788'

// The var overrides (handed to `cf dev` as `WORKER_VAR_OVERRIDES`, see
// `crates/live-share-worker/cloudflare.config.ts`) that point the worker's GitHub calls
// at the mock server (`e2e/mock-github-oauth-server.ts`) instead of real
// GitHub, using the var names the worker itself publishes
// (`e2eWorkerConfig.json`, from `crates/live-share-worker/src/e2e_worker_config.rs`).
// The session TTL of `0` disables the worker's hour-long cache of a token's
// last successful GitHub check, so a token the mock has revoked is rejected
// on its very next use, as it would be in prod once that cache expires. The
// reset var enables the worker's test-only `POST /e2e/reset` route (see
// `crates/live-share-worker/src/e2e_reset.rs`), allowlisting every synthetic
// account an e2e scenario signs in as -- `e2e/global-setup.ts` calls it to
// wipe their rows left over from the previous local run. Its value is a JSON
// array.
const workerVars = {
  [e2eWorkerConfig.githubClientIdVar]: E2E_GITHUB_CLIENT_ID,
  [e2eWorkerConfig.githubUserUrlVar]: `${MOCK_GITHUB_ORIGIN}/user`,
  [e2eWorkerConfig.githubTokenUrlVar]: `${MOCK_GITHUB_ORIGIN}/login/oauth/access_token`,
  [e2eWorkerConfig.githubTokenRevocationUrlVar]: `${MOCK_GITHUB_ORIGIN}/applications/${e2eWorkerConfig.clientIdPlaceholder}/token`,
  [e2eWorkerConfig.sessionTtlMillisVar]: '0',
  [e2eWorkerConfig.resetGithubUserIdsVar]: JSON.stringify(
    Object.values(KNOWN_GITHUB_USER_IDS),
  ),
}

type WebServer = Extract<
  NonNullable<PlaywrightTestConfig['webServer']>,
  unknown[]
>[number]

export const webServers: WebServer[] = [
  {
    // Skip `predev` (the cargo-component/jco build) since pkg-component is
    // already built; just start Vite. `--strictPort` so Playwright's `url`
    // check below fails fast instead of hanging if 5183 is somehow taken,
    // rather than Vite silently falling back to the next free port.
    command: 'pnpm exec vite --port 5183 --strictPort',
    url: 'http://localhost:5183',
    reuseExistingServer: true,
    timeout: 60_000,
    env: {
      // Redirects Synced Share's owner/viewer fetches (scheme picked by
      // `syncedShareWorkerOrigin`, see `web/src/syncedShare/workerUrl.ts`)
      // at the local `live-share-worker` below instead of the real,
      // deployed one — otherwise every Synced Share scenario would burn
      // real Cloudflare D1 writes/reads on every e2e run.
      VITE_SYNCED_SHARE_HOST: 'localhost:8797',
    },
  },
  {
    // Mock of the two GitHub HTTP endpoints the Synced Share worker calls
    // server-side (token exchange + `GET /user`) -- see
    // `e2e/mock-github-oauth-server.ts`'s own doc comment for why this
    // has to be a real local server rather than `page.route()`
    // interception (those calls happen inside the `cf dev` process
    // below, not the browser). Ensures no Synced Share e2e run ever makes
    // a real GitHub API call (task 11).
    command: 'node e2e/mock-github-oauth-server.ts',
    url: 'http://localhost:8788/health',
    reuseExistingServer: true,
    timeout: 15_000,
  },
  {
    // Plain HTTP (no `--local-protocol https`): `syncedShareWorkerOrigin`
    // (`web/src/syncedShare/workerUrl.ts`) only uses `https://` for the
    // real `*.workers.dev` host, plain `http://` for localhost -- avoids
    // every new browser profile/device needing to trust cf dev's
    // self-signed cert before Synced Share e2e scenarios (or manual local
    // testing, see `dekit.yaml`) can reach it. Runs against Miniflare's
    // local D1 emulation (no real Cloudflare account involved), so it's
    // free to hit as often as the suite wants and needs no `cf
    // auth login`. Runs the new Rust worker (`crates/live-share-worker`, task
    // 11 retired the old TypeScript/KV one) -- migrations are applied
    // first since Miniflare's local D1 starts empty (idempotent: a no-op
    // once already applied, so this is cheap on every subsequent run).
    // The `WORKER_VAR_OVERRIDES` point the worker's GitHub calls at the mock
    // server above instead of real GitHub -- see
    // `crates/live-share-worker/src/oauth.rs` and
    // `src/identity/github.rs` for the env vars they read.
    // `SYNCED_SHARE_SESSION_TTL_MILLIS:0` disables the worker's hour-long
    // cache of a token's last successful GitHub check (see
    // `src/identity.rs`), so a token the mock server has revoked is
    // rejected on its very next use, as it would be in prod once that
    // cache expires. `e2eResetVar` (top of file) enables the test-only
    // `POST /e2e/reset` route `e2e/global-setup.ts` calls.
    //
    // Deliberately NOT port 8787 (`just dev`'s port, see `dekit.yaml`) and
    // deliberately NOT the default `--persist-to` D1 state dir: both are
    // isolated from `just dev`'s own `cf dev` instance so that
    // running e2e while `just dev` is active never attaches to dev's
    // worker (which lacks the `WORKER_VAR_OVERRIDES` above and would send
    // GitHub-sign-in scenarios at real GitHub) or shares/pollutes dev's
    // local D1 data. Both `.wrangler/*` paths are already covered by the
    // repo-root `.gitignore`'s `/crates/live-share-worker/.wrangler`
    // entry.
    //
    // `< /dev/null`: with nothing left to apply, `cf d1 migrations apply`
    // prints `[]` and then never exits while stdin is an open pipe, which
    // would hang the `&&` chain until the timeout.
    // CI applies the migrations in its own step instead (this command can
    // fail to exit after applying there, which would hang the `&&` chain),
    // and there `cf dev` ignores `--persist-to` and uses the default
    // `.wrangler/state`, so CI uses that dir for both.
    command:
      (process.env.CI
        ? ''
        : `pnpm exec cf d1 migrations apply ${deployConfig.d1DatabaseId} --local --dir ../../live-share-worker/migrations --persist-to .wrangler/e2e-state < /dev/null && `) +
      `pnpm exec cf dev --port 8797${process.env.CI ? '' : ' --persist-to .wrangler/e2e-state'}`,
    stdout: 'pipe',
    stderr: 'pipe',
    env: { WORKER_VAR_OVERRIDES: JSON.stringify(workerVars) },
    cwd: '../crates/live-share-worker',
    port: 8797,
    reuseExistingServer: true,
    timeout: process.env.CI ? 300_000 : 60_000,
  },
]
