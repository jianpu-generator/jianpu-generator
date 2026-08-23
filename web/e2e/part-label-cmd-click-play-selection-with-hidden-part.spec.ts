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
 * Regression test: Cmd/Ctrl-clicking a part label to select "every part in
 * that system" (see `part-label-cmd-click-selects-whole-system.spec.ts`) and
 * then pressing "play selection" plays the WRONG parts whenever a part
 * earlier in the declaration order is hidden.
 *
 * Root cause: `useNoteSelection.ts`'s `selectedNoteRangePlaybackInfo` resolves
 * each selected run's part name via `parts[partIndex]`, where `partIndex`
 * (`run.sourcePartIndex`) comes from `noteSpans` — fetched via the
 * `listNoteSpans` worker message *with* `enabledTracks`, so hidden parts are
 * `Vec::retain`d out and every later part's index is compacted down. `parts`
 * itself comes from the `listParts` worker message, sent with no
 * `enabledTracks` at all, so it stays the full, unfiltered, declaration-order
 * list. Looking a compacted index up in the uncompacted array resolves to the
 * wrong part whenever anything before the selected part is hidden.
 *
 * Fixture: three parts, Melody / Harmony / Bass. Harmony (the middle part) is
 * hidden, which compacts Bass from source part-index 2 down to rendered
 * part-index 1. Cmd/Ctrl-clicking Melody's label selects every visible part
 * in the system — Melody and Bass — so a correct "play selection" mutes
 * everything except Melody and Bass. The bug instead resolves the
 * compacted-index-1 run to Harmony (`parts[1]` in the unfiltered array),
 * so playback mutes Bass — a part the user actually selected — and unmutes
 * Harmony, a part that's hidden and was never selected at all.
 */
const source = [
  '# metadata',
  'title = "part label cmd click play selection hidden part test"',
  'max_measures_per_system = 1',
  '',
  '# parts',
  'Melody [M] = notes',
  'Harmony [H] = notes',
  'Bass [B] = notes',
  '',
  '# score',
  '[M] 1 2', // measure 0
  '[H] 5', // 1 note/measure
  '[B] 3 4', // 2 notes/measure
].join('\n')

async function loadFixture(page: import('@playwright/test').Page) {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'part-label-cmd-click-play-hidden-part-test.jianpu',
        userFiles: { 'part-label-cmd-click-play-hidden-part-test.jianpu': src },
        bin: {},
        fileIds: {
          'part-label-cmd-click-play-hidden-part-test.jianpu':
            'part-label-cmd-click-play-hidden-part-test-id-001',
        },
      }),
    )
  }, source)
}

/** Patches `Worker.prototype.postMessage` before the app's worker is
 * created, capturing the `enabledTracks` of the most recent
 * `generateMeasureRangeAudio` request on `window.__lastEnabledTracks`. This
 * is the exact payload `useMeasureAudioPlayback.playMeasureRange` sends (see
 * `useMeasureAudioPlayback.ts`), computed synchronously on click — reading it
 * back exposes the resolved part names without waiting on the soundfont
 * decode or real-time `<audio>` playback. */
async function captureEnabledTracks(page: import('@playwright/test').Page) {
  await page.addInitScript(() => {
    const win = window as typeof window & {
      __lastEnabledTracks?: string[]
    }
    const origPostMessage = Worker.prototype.postMessage
    Worker.prototype.postMessage = function (
      this: Worker,
      message: unknown,
      ...rest: unknown[]
    ) {
      if (
        message &&
        typeof message === 'object' &&
        (message as { type?: string }).type === 'generateMeasureRangeAudio'
      ) {
        win.__lastEnabledTracks = (
          message as { enabledTracks?: string[] }
        ).enabledTracks
      }
      // biome-ignore lint/suspicious/noExplicitAny: forwarding to the native postMessage overload
      return (origPostMessage as any).call(this, message, ...rest)
    }
  })
}

/** Waits for measureSpans to be primed (same priming dance the measure-select
 * specs use) so the SVG has settled before hit-testing. */
async function primeMeasureSpans(page: import('@playwright/test').Page) {
  await focusEditor(page)
  await page.keyboard.press('Control+g')
  await page.keyboard.type('11')
  await page.keyboard.press('Enter')
  await expect(page.locator('button.play-measure-btn')).toHaveText(/Measure/, {
    timeout: 5_000,
  })
  await expect(
    page.locator('.preview-page [data-testid="measure-highlight"]').first(),
  ).toBeVisible({ timeout: 5_000 })
}

test('playing a Cmd/Ctrl-click part-label selection only enables the visible selected parts, even when another part is hidden', async ({
  page,
}) => {
  test.setTimeout(75_000)

  await captureEnabledTracks(page)
  await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector(
    '[data-tag="part-label"][data-part-index="0"][data-measure-index-start="0"]',
    { timeout: 10_000 },
  )

  // Hide Harmony — the middle part — so Bass compacts from rendered
  // part-index 2 down to 1.
  const harmonyPill = page.locator('.part-toggle-pill').filter({
    has: page.locator('.part-toggle-abbr', { hasText: /^H$/ }),
  })
  await harmonyPill.locator('.part-toggle-segment--eye').click()

  const noteRects = page.locator('rect[data-variant="note-click-target-rect"]')
  // Melody (2) + Bass (2), Harmony hidden = 4 rendered notes.
  await expect(noteRects).toHaveCount(4, { timeout: 10_000 })
  // Give the debounced listNoteSpans worker round-trip time to catch up
  // with the new enabledTracks before selecting.
  await page.waitForTimeout(2000)

  await primeMeasureSpans(page)

  const melodyLabel = page.locator(
    '[data-tag="part-label"][data-part-index="0"][data-measure-index-start="0"]',
  )
  await expect(melodyLabel).toBeVisible({ timeout: 5_000 })

  const box = await melodyLabel.boundingBox()
  if (!box) throw new Error('Could not get bounding box for Melody label.')

  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.keyboard.down('Control')
  await page.mouse.down()
  await page.mouse.up()
  await page.keyboard.up('Control')

  // Sanity check: the selection itself (already-fixed bug, see
  // `measure-drag-selects-notes-with-hidden-part.spec.ts`) correctly covers
  // both visible parts — Melody (compacted index 0) and Bass (compacted
  // index 1) — and nothing else.
  await expect(
    page.locator(
      '[data-tag="note"][data-note-drag-selected][data-part-index="0"]',
    ),
  ).toHaveCount(2)
  await expect(
    page.locator(
      '[data-tag="note"][data-note-drag-selected][data-part-index="1"]',
    ),
  ).toHaveCount(2)
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(4)

  const playBtn = page.locator('button.play-measure-btn')
  await expect(playBtn).toHaveText(/Selection/, { timeout: 5_000 })
  await expect(playBtn).toBeEnabled({ timeout: 30_000 })

  await playBtn.click()

  await page.waitForFunction(
    () =>
      (window as typeof window & { __lastEnabledTracks?: string[] })
        .__lastEnabledTracks !== undefined,
    { timeout: 10_000 },
  )
  const enabledTracks = await page.evaluate(
    () =>
      (window as typeof window & { __lastEnabledTracks?: string[] })
        .__lastEnabledTracks,
  )

  // The user selected Melody and Bass (both visible) — playback must enable
  // exactly those, never Harmony (hidden, and never selected at all).
  expect(enabledTracks).not.toBeUndefined()
  expect([...(enabledTracks ?? [])].sort()).toEqual(['B', 'M'])
})
