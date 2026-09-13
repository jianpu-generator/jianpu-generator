import { defineConfig, devices } from '@playwright/test'
import { defineBddConfig } from 'playwright-bdd'

// Fully separate from `playwright.config.ts` (distinct `webServer`/
// `baseURL`) -- the two must never be conflated. The main suite always
// serves the app from Vite's dev server at the domain root and mocks
// GitHub's real server; this config instead serves the actual *built*,
// base-pathed `web/dist` artifact the way GitHub Pages actually serves it
// (static files + a real-404-status `404.html` fallback), to catch a
// static-hosting/base-path regression like the one
// `github-callback-production-routing.feature` documents. Nothing about
// GitHub OAuth itself is mocked here since the scenario only cares whether
// the *page* loads at that URL, not whether sign-in completes.
const testDir = defineBddConfig({
  features: 'e2e-pages/features/**/*.feature',
  steps: 'e2e-pages/features/steps/**/*.ts',
  // A distinct `outputDir` from the main suite's default (`.features-gen`,
  // see `playwright.config.ts`) -- without this, both configs generate
  // into the same shared directory, and running this config would also
  // pick up (and try to execute against *this* config's webServer/baseURL)
  // any leftover generated spec files from a previous main-suite run.
  outputDir: '.features-gen-pages',
})

export default defineConfig({
  testDir,
  fullyParallel: true,
  retries: 0,
  use: {
    baseURL: 'http://localhost:4174/jianpu-generator/',
  },
  webServer: {
    command: 'node e2e-pages/static-pages-server.mjs',
    url: 'http://localhost:4174/jianpu-generator/',
    reuseExistingServer: true,
    timeout: 15_000,
    env: {
      DIST_DIR: 'dist',
      BASE_PATH: '/jianpu-generator/',
      PORT: '4174',
    },
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
})
