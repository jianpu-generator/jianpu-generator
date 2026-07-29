import { expect, test } from '@playwright/test'

// The soundfont is a real ~30 MB asset; some sandboxed environments fail to
// write Chromium's HTTP disk cache for large responses
// (net::ERR_CACHE_WRITE_FAILURE), which otherwise breaks the fetch entirely.
test.use({
  launchOptions: {
    args: ['--disk-cache-dir=/tmp/chromium-e2e-cache', '--disable-http-cache'],
  },
})

function delayRoute(delayMs: number) {
  return async (route: import('@playwright/test').Route) => {
    await new Promise((resolve) => setTimeout(resolve, delayMs))
    return route.fallback()
  }
}

test('asset loading banner shows soundfont, fonts, and wasm progress, then hides once all are ready', async ({
  page,
}) => {
  test.setTimeout(60_000)

  // Slow down every asset family so each row has a window to be observed
  // before its request resolves. Soundfont is delayed longer than the fonts
  // and the wasm module so we can also see rows disappear independently as
  // each asset finishes, rather than all at once.
  await page.route('**/*.sf2', delayRoute(3_000))
  await page.route('**/*.otf', delayRoute(800))
  await page.route('**/*.ttf', delayRoute(800))
  await page.route('**/*.wasm', delayRoute(800))

  await page.goto('/')

  const soundfontRow = page.getByText('Soundfont (choir audio)')
  const fontsRow = page.getByText('Fonts (PDF export)')
  const wasmRow = page.getByText('WebAssembly module')

  await expect(soundfontRow).toBeVisible()
  await expect(fontsRow).toBeVisible()
  await expect(wasmRow).toBeVisible()

  // Fonts and wasm finish first (shorter delay); their rows disappear while
  // the soundfont row (longer delay) is still showing.
  await expect(fontsRow).toHaveCount(0, { timeout: 10_000 })
  await expect(wasmRow).toHaveCount(0, { timeout: 10_000 })
  await expect(soundfontRow).toBeVisible()

  // Once the soundfont finishes too, the whole banner disappears.
  await expect(soundfontRow).toHaveCount(0, { timeout: 10_000 })

  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
})

test('asset loading banner shows an error state when an asset fails to load', async ({
  page,
}) => {
  await page.route('**/*.sf2', (route) => route.abort())

  await page.goto('/')

  await expect(page.getByText('Soundfont (choir audio)')).toBeVisible()
  await expect(page.getByText('Error')).toBeVisible({ timeout: 10_000 })

  // Fonts and wasm still load fine, so only the soundfont row remains.
  await expect(page.getByText('Fonts (PDF export)')).toHaveCount(0, {
    timeout: 15_000,
  })
  await expect(page.getByText('WebAssembly module')).toHaveCount(0, {
    timeout: 15_000,
  })
  await expect(page.getByText('Soundfont (choir audio)')).toBeVisible()
})
