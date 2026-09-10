import { defineConfig, devices } from '@playwright/test'
import { defineBddConfig } from 'playwright-bdd'
import { CHROMIUM_CACHE_DIR_PREFIX } from './e2e/chromiumCachePrefix'

const testDir = defineBddConfig({
  features: 'e2e/features/**/*.feature',
  steps: 'e2e/features/steps/**/*.ts',
})

export default defineConfig({
  testDir,
  fullyParallel: true,
  // Cleans up the per-worker Chromium cache dirs created below (via
  // `--disk-cache-dir`) after the run finishes. Runs regardless of whether
  // tests pass, fail, or time out.
  globalTeardown: './e2e/global-teardown.ts',
  // No in-run retries: a flaky test masked here would just report "passed"
  // with no record that it ever failed. Flakiness is instead resolved across
  // whole-suite passes by scripts/resolve-e2e-flakes.mjs (see
  // `test:e2e:resolve`), which reruns only the tests still failing after
  // each pass until the same set fails 3 times in a row.
  retries: 0,
  use: {
    baseURL: 'http://localhost:5173',
  },
  webServer: [
    {
      // Skip `predev` (the cargo-component/jco build) since pkg-component is
      // already built; just start Vite.
      command: 'pnpm exec vite',
      url: 'http://localhost:5173',
      reuseExistingServer: true,
      timeout: 60_000,
      env: {
        // Redirects Synced Share's owner/viewer fetches (scheme picked by
        // `syncedShareWorkerOrigin`, see `web/src/syncedShare/workerUrl.ts`)
        // at the local `live-share-worker` below instead of the real,
        // deployed one — otherwise every Synced Share scenario would burn
        // real Cloudflare D1 writes/reads on every e2e run.
        VITE_SYNCED_SHARE_HOST: 'localhost:8787',
      },
    },
    {
      // Mock of the two GitHub HTTP endpoints the Synced Share worker calls
      // server-side (token exchange + `GET /user`) -- see
      // `e2e/mock-github-oauth-server.mjs`'s own doc comment for why this
      // has to be a real local server rather than `page.route()`
      // interception (those calls happen inside the `wrangler dev` process
      // below, not the browser). Ensures no Synced Share e2e run ever makes
      // a real GitHub API call (task 11).
      command: 'node e2e/mock-github-oauth-server.mjs',
      url: 'http://localhost:8788/health',
      reuseExistingServer: true,
      timeout: 15_000,
    },
    {
      // Plain HTTP (no `--local-protocol https`): `syncedShareWorkerOrigin`
      // (`web/src/syncedShare/workerUrl.ts`) only uses `https://` for the
      // real `*.workers.dev` host, plain `http://` for localhost -- avoids
      // every new browser profile/device needing to trust wrangler dev's
      // self-signed cert before Synced Share e2e scenarios (or manual local
      // testing, see `dekit.yaml`) can reach it. Runs against Miniflare's
      // local D1 emulation (no real Cloudflare account involved), so it's
      // free to hit as often as the suite wants and needs no `wrangler
      // login`. Runs the new Rust worker (`crates/live-share-worker`, task
      // 11 retired the old TypeScript/KV one) -- migrations are applied
      // first since Miniflare's local D1 starts empty (idempotent: a no-op
      // once already applied, so this is cheap on every subsequent run).
      // The `--var` overrides point the worker's GitHub calls at the mock
      // server above instead of real GitHub -- see
      // `crates/live-share-worker/src/oauth.rs` and
      // `src/identity/github.rs` for the env vars they read.
      command:
        'npx wrangler d1 migrations apply DB --local && ' +
        'npx wrangler dev --port 8787 ' +
        '--var SYNCED_SHARE_GITHUB_USER_URL:http://localhost:8788/user ' +
        '--var SYNCED_SHARE_GITHUB_TOKEN_URL:http://localhost:8788/login/oauth/access_token',
      cwd: '../crates/live-share-worker',
      port: 8787,
      reuseExistingServer: true,
      timeout: 60_000,
    },
  ],
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        launchOptions: {
          args: [
            '--autoplay-policy=no-user-gesture-required',
            // Several tests load real, large assets (soundfonts, PDF fonts).
            // Some sandboxed environments fail to write Chromium's HTTP disk
            // cache for large responses (net::ERR_CACHE_WRITE_FAILURE),
            // which otherwise breaks those fetches entirely. Applied to
            // every test (not just the affected ones) since playwright-bdd's
            // generated spec files don't support a per-scenario
            // `test.use({ launchOptions })` override, and the flags are
            // harmless for tests that don't hit large assets.
            //
            // Each worker runs in its own process, so `process.pid` gives
            // every worker's Chromium instance a distinct cache dir — with
            // `fullyParallel`, workers previously shared one dir and
            // stomped on each other's cache writes/reads, causing
            // intermittent slowdowns and timeouts under parallel runs.
            `--disk-cache-dir=${CHROMIUM_CACHE_DIR_PREFIX}${process.pid}`,
            '--disable-http-cache',
          ],
        },
      },
    },
  ],
})
