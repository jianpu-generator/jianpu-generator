import { existsSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import type { Response } from '@playwright/test'
import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

// `import.meta.url`, not `__dirname` -- `web/package.json` sets
// `"type": "module"`, so this file (and playwright-bdd's loader for it)
// runs as native ESM, where `__dirname` doesn't exist.
const __dirname = dirname(fileURLToPath(import.meta.url))
const DIST_DIR = join(__dirname, '../../../dist')

Given('the production build with its GitHub-Pages 404 fallback exists', () => {
  if (
    !existsSync(join(DIST_DIR, 'index.html')) ||
    !existsSync(join(DIST_DIR, '404.html'))
  ) {
    throw new Error(
      'web/dist/index.html and web/dist/404.html must both exist first -- ' +
        'run `pnpm run test:e2e:pages` (which builds them), not this ' +
        'Playwright config directly.',
    )
  }
})

let response: Response | null = null

When(
  'a browser navigates directly to the GitHub callback path with a code and state',
  async ({ page }) => {
    response = await page.goto(
      'synced-share/github-callback?code=e2e-fake-code&state=e2e-fake-state',
    )
  },
)

Then(
  'the response is a GitHub-Pages-style 404 that still renders the callback page',
  async ({ page }) => {
    expect(response?.status()).toBe(404)
    await expect(
      page.getByTestId('synced-share-github-callback-page'),
    ).toBeVisible()
  },
)
