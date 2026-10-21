import { defineConfig, devices } from '@playwright/test'
import { defineBddConfig } from 'playwright-bdd'
import { CHROMIUM_CACHE_DIR_PREFIX } from './e2e/chromiumCachePrefix'
import { E2E_EMAILS } from './e2e/e2eAccounts.ts'
import deployConfig from './src/generated/live-share-worker/deployConfig.json' with {
  type: 'json',
}
import e2eWorkerConfig from './src/generated/live-share-worker/e2eWorkerConfig.json' with {
  type: 'json',
}

// The var overrides (handed to `cf dev` as `WORKER_VAR_OVERRIDES`, see
// `crates/live-share-worker/cloudflare.config.ts`), using the var names the
// worker itself publishes (`e2eWorkerConfig.json`, from
// `crates/live-share-worker/src/e2e_worker_config.rs`). The reset var
// enables the worker's test-only `POST /e2e/reset` route (see
// `crates/live-share-worker/src/e2e_reset.rs`), allowlisting every synthetic
// account an e2e scenario signs in as -- `e2e/global-setup.ts` calls it to
// wipe their rows left over from the previous local run, and some scenarios
// call it to start from a fresh account. Its value is a JSON array.
const workerVars = {
  [e2eWorkerConfig.resetEmailsVar]: JSON.stringify(E2E_EMAILS),
}

// Raises the worker's per-IP sign-in and sign-up rate limits (see
// `cloudflare.config.ts`): every parallel e2e worker signs in from the same
// IP, many times a minute.
const E2E_RATE_LIMIT_PER_MINUTE = '100000'

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
  // Wipes the synthetic e2e accounts' leftover cloud rows. Playwright
  // starts `webServer` (below) before running this, so the worker is up.
  globalSetup: './e2e/global-setup.ts',
  // No in-run retries: a flaky test masked here would just report "passed"
  // with no record that it ever failed. Flakiness is instead resolved across
  // whole-suite passes by scripts/resolve-e2e-flakes.ts (see
  // `test:e2e:resolve`), which reruns only the tests still failing after
  // each pass until the same set fails 3 times in a row.
  retries: 0,
  use: {
    // Deliberately NOT `just dev`'s port (5173, see `dekit.yaml`): with
    // `reuseExistingServer: true` below, running e2e while `just dev` is
    // active would otherwise silently attach to dev's Vite/worker instead
    // of spawning e2e's own — and dev's `cf dev` (see the last webServer
    // entry) has neither the e2e reset route nor the raised rate limits. A
    // dedicated port means e2e always starts (or reuses) its own instance,
    // regardless of whether `just dev` happens to be running.
    baseURL: 'http://localhost:5183',
  },
  webServer: [
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
      // `WORKER_VAR_OVERRIDES` (top of file) enables the test-only
      // `POST /e2e/reset` route `e2e/global-setup.ts` calls, and
      // `RATE_LIMIT_PER_MINUTE_OVERRIDE` raises the sign-in/sign-up rate
      // limits.
      //
      // Deliberately NOT port 8787 (`just dev`'s port, see `dekit.yaml`) and
      // deliberately NOT the default `--persist-to` D1 state dir: both are
      // isolated from `just dev`'s own `cf dev` instance so that
      // running e2e while `just dev` is active never attaches to dev's
      // worker (which lacks the overrides above) or shares/pollutes dev's
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
      env: {
        WORKER_VAR_OVERRIDES: JSON.stringify(workerVars),
        RATE_LIMIT_PER_MINUTE_OVERRIDE: E2E_RATE_LIMIT_PER_MINUTE,
      },
      cwd: '../crates/live-share-worker',
      port: 8797,
      reuseExistingServer: true,
      timeout: process.env.CI ? 300_000 : 60_000,
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
