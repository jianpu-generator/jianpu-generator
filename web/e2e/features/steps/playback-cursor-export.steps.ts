import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

/**
 * Regression coverage for the WAV/MP3 playback-cursor parity gap: WAV's
 * inline player animates the playback cursor onto the sounding note, but
 * MP3's never has, even though `usePlaybackCursor` itself doesn't care which
 * codec produced the audio (see `playback-cursor-export.feature`'s header
 * comment for the root cause). Parameterized over both formats so a fix
 * that special-cases one of them, or a regression in the WAV path, both
 * show up here.
 *
 * Fixture: a single part, single note — the smallest score that can show
 * (or fail to show) a cursor highlight at all.
 */
const source = [
  '# metadata',
  'title = "playback cursor export test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '[M] 1',
].join('\n')

Given('the playback-cursor export test fixture is loaded', async ({ page }) => {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'playback-cursor-export-test.jianpu',
        userFiles: { 'playback-cursor-export-test.jianpu': src },
        bin: {},
        fileIds: {
          'playback-cursor-export-test.jianpu':
            'playback-cursor-export-test-id-001',
        },
      }),
    )
  }, source)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('[data-tag="measure"][data-measure-index="0"]', {
    timeout: 10_000,
  })
})

Then(
  'the inline audio player is visible with a blob src, as seen in playback cursor export',
  async ({ page }) => {
    const audioPlayer = page.locator('.preview-audio-player')
    await expect(audioPlayer).toBeVisible({ timeout: 15_000 })
    await expect(audioPlayer).toHaveAttribute('src', /^blob:/)
  },
)

When('I play the inline audio player', async ({ page }) => {
  const audioPlayer = page.locator('.preview-audio-player')
  await audioPlayer.evaluate((el: HTMLAudioElement) => el.play())
})

Then('the first note shows the playback cursor highlight', async ({ page }) => {
  await expect(
    page.locator(
      '[data-tag="note"][data-part-index="0"][data-note-id="0"] rect[data-variant="playback-cursor-rect"]',
    ),
  ).toHaveAttribute('fill', 'rgba(220,38,38,0.25)', { timeout: 20_000 })
})
