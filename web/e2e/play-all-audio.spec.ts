import { expect, test } from '@playwright/test'

// The soundfont is a real ~30 MB asset; some sandboxed environments fail to
// write Chromium's HTTP disk cache for large responses
// (net::ERR_CACHE_WRITE_FAILURE), which otherwise breaks the fetch entirely.
test.use({
  launchOptions: {
    args: ['--disk-cache-dir=/tmp/chromium-e2e-cache', '--disable-http-cache'],
  },
})

const SINGLE_PART_SOURCE = [
  '# metadata',
  'title = "Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

async function loadSource(
  page: import('@playwright/test').Page,
  source: string,
) {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'test.jianpu',
        userFiles: { 'test.jianpu': src },
        bin: {},
        fileIds: { 'test.jianpu': crypto.randomUUID() },
      }),
    )
  }, source)
}

/**
 * Exercises the "Play All" button on a small fixed score, mirroring
 * `export-wav-toast.spec.ts`'s source-loading pattern: clicking it generates
 * and autoplays audio for the whole score, asserting on the button's
 * visible "playing" state rather than just "didn't crash". Mirrors
 * `play-measure-audio.spec.ts`.
 */
test('clicking Play All starts and finishes playback of the whole score', async ({
  page,
}) => {
  test.setTimeout(90_000)

  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  const playAllBtn = page.locator('button.play-all-btn')
  await expect(playAllBtn).toBeVisible({ timeout: 15_000 })

  // The button stays disabled until the soundfont (a real ~30 MB asset)
  // finishes loading; wait for that instead of asserting a fixed delay.
  await expect(playAllBtn).toBeEnabled({ timeout: 30_000 })

  await playAllBtn.click()

  // Playback engaged: button switches to the pause/playing variant. Audio
  // synthesis for the whole score takes noticeably longer than a single
  // measure (see `play-measure-audio.spec.ts`), so this allows more time.
  await expect(playAllBtn).toHaveClass(/play-all-btn--playing/, {
    timeout: 15_000,
  })

  // This fixed score (four quarter notes) is short — playback should finish
  // and the button should revert to its normal (non-playing) state on its
  // own, without the user pausing it. Generous timeout: real-time `<audio>`
  // playback duration is sensitive to CPU throttling/contention in
  // sandboxed/CI runners, so wall-clock completion can lag well past the
  // audio's nominal duration (see FLAKY_TESTS.md).
  await expect(playAllBtn).not.toHaveClass(/play-all-btn--playing/, {
    timeout: 45_000,
  })
})
