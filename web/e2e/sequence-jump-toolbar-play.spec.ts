import { expect, test } from '@playwright/test'

// The soundfont is a real ~30 MB asset; some sandboxed environments fail to
// write Chromium's HTTP disk cache for large responses
// (net::ERR_CACHE_WRITE_FAILURE), which otherwise breaks the fetch entirely.
// `test.use({ launchOptions })` forces a new worker and must stay top-level
// in the file (mirrors play-measure-audio.spec.ts).
test.use({
  launchOptions: {
    args: ['--disk-cache-dir=/tmp/chromium-e2e-cache', '--disable-http-cache'],
  },
})

/**
 * Same `# sequence` source as sequence-jump-toolbar.spec.ts — see that file
 * for the annotated line breakdown.
 */
const source = [
  '# metadata',
  'title = "test"',
  '',
  '# parts',
  'M = notes',
  '',
  '# sequence',
  'A, B, B',
  '',
  '# score',
  'time=4/4 key=C4 bpm=120 label="A"',
  '1 2 3 4',
  '',
  'label="B"',
  "5 6 7 1'",
].join('\n')

test.beforeEach(async ({ page }) => {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'sequence-test.jianpu',
        userFiles: { 'sequence-test.jianpu': src },
        bin: {},
        fileIds: { 'sequence-test.jianpu': crypto.randomUUID() },
      }),
    )
  }, source)

  await page.goto('/')
  await page.waitForSelector(
    '[data-testid="play-from-current-measure-button"]',
    { timeout: 15_000 },
  )
})

test('clicking play on a selected sequence entry starts and finishes playback', async ({
  page,
}) => {
  test.setTimeout(75_000)

  const buttons = page
    .locator('[role="toolbar"]')
    .nth(1)
    .locator('button.section-jump-btn')
  await expect(buttons).toHaveCount(3, { timeout: 15_000 })
  await buttons.nth(0).click()

  const playBtn = page.getByTestId('play-from-current-measure-button')
  await expect(playBtn).toHaveAttribute(
    'aria-label',
    'Play sequence from Measure 1',
  )
  // The button stays disabled until the soundfont (a real ~30 MB asset)
  // finishes loading; wait for that instead of asserting a fixed delay.
  await expect(playBtn).toBeEnabled({ timeout: 30_000 })

  await playBtn.click()

  await expect(playBtn).toHaveClass(/play-from-measure-btn--playing/, {
    timeout: 5_000,
  })

  // Measure 0 ("1 2 3 4") is short — playback should finish and the button
  // should revert to its normal (non-playing) state on its own. Generous
  // timeout: real-time `<audio>` playback duration is sensitive to CPU
  // throttling/contention in sandboxed/CI runners, so wall-clock completion
  // can lag well past the audio's nominal duration (see FLAKY_TESTS.md).
  await expect(playBtn).not.toHaveClass(/play-from-measure-btn--playing/, {
    timeout: 30_000,
  })
})
