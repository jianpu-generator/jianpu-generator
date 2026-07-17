import { expect, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

// The soundfont is a real ~30 MB asset; some sandboxed environments fail to
// write Chromium's HTTP disk cache for large responses
// (net::ERR_CACHE_WRITE_FAILURE), which otherwise breaks the fetch entirely.
test.use({
  launchOptions: {
    args: ['--disk-cache-dir=/tmp/chromium-e2e-cache', '--disable-http-cache'],
  },
})

/**
 * `cmd-enter-play.spec.ts` only covers the no-op case (no measure selected).
 * This test exercises the real playback path: a measure is selected, the
 * soundfont has loaded, and clicking play actually starts audio — asserting
 * on the play button's visible "playing" state rather than just "didn't
 * crash".
 *
 * Uses the default demo file (demo/00-header.jianpu); line 16 is the first
 * note line ("[M] 0 0 0 0" → measure 0), as established by
 * `measure-label.spec.ts`.
 */
test('clicking play on a selected measure starts and finishes playback', async ({
  page,
}) => {
  test.setTimeout(60_000)

  await page.goto('/')
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })

  // Focus the Monaco editor and place the cursor inside measure 0.
  await focusEditor(page)
  await page.keyboard.press('Control+g')
  await page.keyboard.type('16')
  await page.keyboard.press('Enter')

  const playBtn = page.locator('button.play-measure-btn')

  // Confirm the measure was actually selected (debounce + worker round-trip).
  await expect(playBtn).toHaveText(/Measure/, { timeout: 5_000 })

  // The button stays disabled until the soundfont (a real ~30 MB asset)
  // finishes loading; wait for that instead of asserting a fixed delay.
  await expect(playBtn).toBeEnabled({ timeout: 30_000 })

  await playBtn.click()

  // Playback engaged: button switches to the pause/playing variant.
  await expect(playBtn).toHaveClass(/play-measure-btn--playing/, {
    timeout: 5_000,
  })

  // Measure 0 ("[M] 0 0 0 0", all rests) is short — playback should finish
  // and the button should revert to its normal (non-playing) state on its
  // own, without the user pausing it.
  await expect(playBtn).not.toHaveClass(/play-measure-btn--playing/, {
    timeout: 10_000,
  })
})
