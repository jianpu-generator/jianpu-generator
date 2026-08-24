import { expect, test } from '@playwright/test'
import { gotoShareUrl } from './shareUrlHelper'

const SHARED_FILENAME = 'shared-mobile-test.jianpu'
const SHARED_SOURCE = [
  '# metadata',
  'title = "Shared Mobile Test"',
  '',
  '# parts',
  'Melody = notes',
  '',
  '# score',
  '(time=4/4 key=C4 bpm=120)',
  '1 2 3 4',
].join('\n')

test.use({ viewport: { width: 375, height: 700 } })

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.clear()
  })
})

test('app header scrolls horizontally instead of wrapping when viewing a shared score on a mobile viewport', async ({
  page,
}) => {
  await gotoShareUrl(page, SHARED_FILENAME, SHARED_SOURCE)

  await expect(page.locator('.shared-preview-banner')).toContainText(
    SHARED_FILENAME,
  )

  const header = page.locator('.app-header')

  const { scrollWidth, clientHeight, overflowY } = await header.evaluate(
    (el) => ({
      scrollWidth: el.scrollWidth,
      clientHeight: el.getBoundingClientRect().height,
      overflowY: getComputedStyle(el).overflowY,
    }),
  )

  // On a 375px-wide viewport the header's content (title, shared-preview
  // banner, playback buttons, export controls) is wider than the viewport,
  // so it only fits without wrapping if it overflows horizontally instead.
  expect(scrollWidth).toBeGreaterThan(375)
  // A single, non-wrapped row stays well under the height a two-row wrap
  // would produce.
  expect(clientHeight).toBeLessThan(60)
  // The header must scroll horizontally only. `overflow-x: auto` alone
  // implicitly computes `overflow-y` to `auto` too (per the CSS overflow
  // spec), so any sub-pixel content/box height mismatch makes it vertically
  // scrollable as well unless `overflow-y` is pinned to `hidden`.
  expect(overflowY).toBe('hidden')
})
