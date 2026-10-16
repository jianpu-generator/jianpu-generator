import { defineConfig, devices } from '@playwright/test'
import { defineBddConfig } from 'playwright-bdd'
import { CHROMIUM_CACHE_DIR_PREFIX } from './e2e/chromiumCachePrefix'
import { webServers } from './e2e/webServers.ts'

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
  // each pass until the same set fails several times in a row.
  retries: 0,
  use: {
    // Deliberately NOT `just dev`'s port (5173, see `dekit.yaml`): with
    // `reuseExistingServer: true` below, running e2e while `just dev` is
    // active would otherwise silently attach to dev's Vite/worker instead
    // of spawning e2e's own — and dev's `cf dev` (see next webServer
    // entry) doesn't carry the `SYNCED_SHARE_GITHUB_*_URL` overrides
    // e2e needs to redirect GitHub calls at the mock server below, so any
    // GitHub-sign-in scenario would silently try to hit real GitHub. A
    // dedicated port means e2e always starts (or reuses) its own instance,
    // regardless of whether `just dev` happens to be running.
    baseURL: 'http://localhost:5183',
  },
  webServer: webServers,
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
