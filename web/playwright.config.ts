import { defineConfig, devices } from '@playwright/test'

export default defineConfig({
  testDir: './e2e',
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
          args: ['--autoplay-policy=no-user-gesture-required'],
        },
      },
    },
  ],
})
