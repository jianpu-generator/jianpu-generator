import type { Page } from '@playwright/test'

// `encodeShareHashSuffix` now compresses via brotli-over-WASM, which needs a
// real browser fetch to load the .wasm binary — it can't run in the Node-side
// Playwright test process. Compute it inside the page instead.
export async function encodeShareHashOnPage(
  page: Page,
  filename: string,
  content: string,
): Promise<string> {
  return page.evaluate(
    async ({ filename, content }) => {
      const { encodeShareHashSuffix } = await import('/src/shareUrl.ts')
      return encodeShareHashSuffix(filename, content)
    },
    { filename, content },
  )
}

// Navigates directly to a share URL for `filename`/`content`. A hash-only
// change from the current document is a same-document navigation and won't
// remount the app, so this always forces a full navigation via a blank
// interstitial page first.
export async function gotoShareUrl(
  page: Page,
  filename: string,
  content: string,
): Promise<void> {
  await page.goto('http://localhost:5173/')
  const hash = await encodeShareHashOnPage(page, filename, content)
  await page.goto('about:blank')
  await page.goto(`http://localhost:5173/#share=${hash}`)
}
