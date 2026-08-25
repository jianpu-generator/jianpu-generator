import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

function delayRoute(delayMs: number) {
  return async (route: import('@playwright/test').Route) => {
    await new Promise((resolve) => setTimeout(resolve, delayMs))
    return route.fallback()
  }
}

Given(
  'delayed asset loading routes for soundfont, fonts, and wasm',
  async ({ page }) => {
    // Slow down every asset family so each row has a window to be observed
    // before its request resolves. Soundfont is delayed longer than the
    // fonts and the wasm module so we can also see rows disappear
    // independently as each asset finishes, rather than all at once.
    await page.route('**/*.sf2', delayRoute(3_000))
    await page.route('**/*.otf', delayRoute(800))
    await page.route('**/*.ttf', delayRoute(800))
    await page.route('**/*.wasm', delayRoute(800))
  },
)

When('the app loads with delayed asset routes', async ({ page }) => {
  await page.goto('/')
})

Then(
  'the soundfont, fonts, and wasm rows are all visible',
  async ({ page }) => {
    const soundfontRow = page.getByText('Soundfont (choir audio)')
    const fontsRow = page.getByText('Fonts (PDF export)')
    const wasmRow = page.getByText('WebAssembly module')

    await expect(soundfontRow).toBeVisible()
    await expect(fontsRow).toBeVisible()
    await expect(wasmRow).toBeVisible()
  },
)

Then(
  'the fonts and wasm rows disappear before the soundfont row',
  async ({ page }) => {
    const soundfontRow = page.getByText('Soundfont (choir audio)')
    const fontsRow = page.getByText('Fonts (PDF export)')
    const wasmRow = page.getByText('WebAssembly module')

    // Fonts and wasm finish first (shorter delay); their rows disappear
    // while the soundfont row (longer delay) is still showing.
    await expect(fontsRow).toHaveCount(0, { timeout: 10_000 })
    await expect(wasmRow).toHaveCount(0, { timeout: 10_000 })
    await expect(soundfontRow).toBeVisible()
  },
)

Then(
  'the soundfont row disappears once it finishes loading',
  async ({ page }) => {
    // Once the soundfont finishes too, the whole banner disappears.
    await expect(page.getByText('Soundfont (choir audio)')).toHaveCount(0, {
      timeout: 10_000,
    })
  },
)

Then('the Monaco editor view-lines become visible', async ({ page }) => {
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
})

Given('the soundfont asset route is aborted', async ({ page }) => {
  await page.route('**/*.sf2', (route) => route.abort())
})

When('the app loads with the soundfont route aborted', async ({ page }) => {
  await page.goto('/')
})

Then('the soundfont row shows an error state', async ({ page }) => {
  await expect(page.getByText('Soundfont (choir audio)')).toBeVisible()
  await expect(page.getByText('Error')).toBeVisible({ timeout: 10_000 })
})

Then(
  'the fonts and wasm rows disappear while only the soundfont row remains',
  async ({ page }) => {
    // Fonts and wasm still load fine, so only the soundfont row remains.
    await expect(page.getByText('Fonts (PDF export)')).toHaveCount(0, {
      timeout: 15_000,
    })
    await expect(page.getByText('WebAssembly module')).toHaveCount(0, {
      timeout: 15_000,
    })
    await expect(page.getByText('Soundfont (choir audio)')).toBeVisible()
  },
)
