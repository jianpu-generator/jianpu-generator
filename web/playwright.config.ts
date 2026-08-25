import { defineConfig, devices } from '@playwright/test'
import { defineBddConfig } from 'playwright-bdd'

const testDir = defineBddConfig({
  features: 'e2e/features/**/*.feature',
  steps: 'e2e/features/steps/**/*.ts',
})

export default defineConfig({
  testDir,
  fullyParallel: true,
  retries: 2,
  use: {
    baseURL: 'http://localhost:5173',
  },
  webServer: {
    // Skip `predev` (wasm-pack) since the pkg is already built; just start Vite.
    command: 'pnpm exec vite',
    url: 'http://localhost:5173',
    reuseExistingServer: true,
    timeout: 60_000,
  },
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
            '--disk-cache-dir=/tmp/chromium-e2e-cache',
            '--disable-http-cache',
          ],
        },
      },
    },
  ],
})
