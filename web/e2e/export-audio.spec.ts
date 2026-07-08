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

const MULTI_PART_SOURCE = [
  '# metadata',
  'title = "Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  'Harmony [H] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
  '5 6 7 1',
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

async function toggleEye(
  page: import('@playwright/test').Page,
  abbreviation: string,
) {
  await page
    .locator('.part-toggle-pill')
    .filter({
      has: page.locator('.part-toggle-abbr', {
        hasText: new RegExp(`^${abbreviation}$`),
      }),
    })
    .locator('.part-toggle-segment--eye')
    .click()
}

test('Generate audio produces a playable inline WAV and Regenerate replaces it', async ({
  page,
}) => {
  test.setTimeout(60_000)

  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  // The button's accessible name comes from its `aria-label`, which only
  // ever reads "Generate audio" / "Regenerate audio" — the transient
  // "Generating…" text is inner content, not exposed via role name. For a
  // score this small, synthesis also completes faster than a poll interval
  // can reliably observe the disabled state, so assert on the end state
  // (the rendered player) rather than the transient one.
  const audioButton = page.getByRole('button', { name: /^Generate audio$/ })
  await expect(audioButton).toBeEnabled({ timeout: 30_000 })

  await audioButton.click()

  const audioPlayer = page.locator('.preview-audio-player')
  await expect(audioPlayer).toBeVisible({ timeout: 15_000 })

  const firstSrc = await audioPlayer.getAttribute('src')
  expect(firstSrc).toMatch(/^blob:/)

  // A silently empty/corrupt WAV would still produce a `blob:` src but have
  // zero duration — assert the browser actually decoded playable audio.
  const firstDuration = await audioPlayer.evaluate(
    (el: HTMLAudioElement) =>
      new Promise<number>((resolve) => {
        if (el.readyState >= 1 && Number.isFinite(el.duration)) {
          resolve(el.duration)
          return
        }
        el.addEventListener('loadedmetadata', () => resolve(el.duration), {
          once: true,
        })
      }),
  )
  expect(firstDuration).toBeGreaterThan(0)

  const regenerateButton = page.getByRole('button', {
    name: /^Regenerate audio$/,
  })
  await expect(regenerateButton).toBeVisible()

  await regenerateButton.click()
  await expect(regenerateButton).toBeEnabled({ timeout: 15_000 })

  // The previous blob URL is revoked and a new one set (useJianpuWorker.ts's
  // setNextWavUrl) — assert the src actually changed rather than being reused.
  await expect
    .poll(() => audioPlayer.getAttribute('src'), { timeout: 15_000 })
    .not.toBe(firstSrc)
  const secondSrc = await audioPlayer.getAttribute('src')
  expect(secondSrc).toMatch(/^blob:/)

  // Editing the source in place (no file switch) must not clear the
  // existing audio or reset the button back to "Generate audio" — wavUrl is
  // only cleared on `activeFile` change, not on `source` change.
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+End')
  await page.keyboard.type(' 5')

  await expect(audioPlayer).toBeVisible()
  await expect(audioPlayer).toHaveAttribute('src', secondSrc as string)
  await expect(
    page.getByRole('button', { name: /^Regenerate audio$/ }),
  ).toBeVisible()
})

test('audio download link filename includes only the enabled parts when a part is hidden', async ({
  page,
}) => {
  test.setTimeout(60_000)

  await loadSource(page, MULTI_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  await toggleEye(page, 'H')

  const audioButton = page.getByRole('button', { name: /^Generate audio$/ })
  await expect(audioButton).toBeEnabled({ timeout: 30_000 })
  await audioButton.click()

  const downloadLink = page.locator('.preview-audio-download')
  await expect(downloadLink).toBeVisible({ timeout: 15_000 })
  await expect(downloadLink).toHaveAttribute('download', 'test (M).wav')
})
