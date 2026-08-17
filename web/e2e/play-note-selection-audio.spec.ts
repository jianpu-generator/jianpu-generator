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
 * Companion to `play-measure-audio.spec.ts` and
 * `note-drag-select-highlight.spec.ts`: the play-measure button (see
 * `PlayMeasureButton.tsx`) is repurposed while a note drag-select is active —
 * it switches to a "Selection" label and, when clicked, plays only the
 * drag-selected notes (`useMeasureAudioPlayback.playNoteSelection`) instead
 * of the measure(s) under the cursor. This exercises the real playback path
 * end to end, not just the label swap.
 *
 * Self-contained source (not a demo file) with a generous "max measures per
 * system" and four single-beat notes in one measure, so all four note
 * click-targets render side by side in one row within the viewport.
 */
const dragTestSource = [
  '# metadata',
  'title = "note drag test"',
  'max_measures_per_system = 48',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '[M] 1 2 3 4', // measure 0
].join('\n')

async function loadDragTestFixture(page: import('@playwright/test').Page) {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'note-drag-test.jianpu',
        userFiles: { 'note-drag-test.jianpu': source },
        bin: {},
        fileIds: { 'note-drag-test.jianpu': 'note-drag-test-id-001' },
      }),
    )
  }, dragTestSource)
}

test('clicking the play-measure button with notes drag-selected plays only the selection', async ({
  page,
}) => {
  test.setTimeout(75_000)

  await loadDragTestFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })

  // Wait for the SVG preview to render note click targets for measure 0.
  await page.waitForSelector('[data-tag="measure"][data-measure-index="0"]', {
    timeout: 10_000,
  })
  const noteRects = page.locator('rect[data-variant="note-click-target-rect"]')
  await expect(noteRects).toHaveCount(4, { timeout: 10_000 })

  await focusEditor(page)

  const playBtn = page.locator('button.play-measure-btn')

  // Before any note drag-select, the button reflects the measure under the
  // cursor, not a selection.
  await expect(playBtn).toHaveText(/Measure/, { timeout: 5_000 })

  // Drag a marquee across the first three notes.
  const box0 = await noteRects.nth(0).boundingBox()
  const box2 = await noteRects.nth(2).boundingBox()
  if (!box0 || !box2) {
    throw new Error(
      'Could not get bounding boxes for notes 0 and 2. ' +
        'Ensure the SVG preview has rendered.',
    )
  }
  await page.mouse.move(box0.x + box0.width / 2, box0.y + box0.height / 2)
  await page.mouse.down()
  await page.mouse.move(box2.x + box2.width / 2, box2.y + box2.height / 2, {
    steps: 10,
  })
  await page.mouse.up()

  // The button is repurposed: label switches to "Selection".
  await expect(playBtn).toHaveText(/Selection/, { timeout: 5_000 })

  // The button stays disabled until the soundfont (a real ~30 MB asset)
  // finishes loading; wait for that instead of asserting a fixed delay.
  await expect(playBtn).toBeEnabled({ timeout: 30_000 })

  await playBtn.click()

  // Playback engaged: button switches to the pause/playing variant, still
  // labeled "Selection".
  await expect(playBtn).toHaveClass(/play-measure-btn--playing/, {
    timeout: 5_000,
  })
  await expect(playBtn).toHaveText(/Selection/)

  // The selection is short — playback should finish and the button should
  // revert to its normal (non-playing) state on its own. Generous timeout:
  // real-time `<audio>` playback duration is sensitive to CPU
  // throttling/contention in sandboxed/CI runners, so wall-clock completion
  // can lag well past the audio's nominal duration (see FLAKY_TESTS.md).
  await expect(playBtn).not.toHaveClass(/play-measure-btn--playing/, {
    timeout: 30_000,
  })
})
