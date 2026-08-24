import { expect, test } from '@playwright/test'

// The soundfont is a real ~30 MB asset; some sandboxed environments fail to
// write Chromium's HTTP disk cache for large responses
// (net::ERR_CACHE_WRITE_FAILURE), which otherwise breaks the fetch entirely.
test.use({
  launchOptions: {
    args: ['--disk-cache-dir=/tmp/chromium-e2e-cache', '--disable-http-cache'],
  },
})

/**
 * Regression test: playing a drag-selected part's notes never moved the
 * playback cursor onto those notes whenever an *earlier-declared* part was
 * hidden.
 *
 * Root cause: Rust's `note_timings_seconds` (see
 * `src/midi/timing_note_timings.rs`) deliberately reports each returned
 * `NoteTiming.source_part_index` as the note's true *written* part index,
 * unaffected by whatever `enabled_tracks` mutes that clip's audio down to —
 * so a muted/repeated clip's `note_id`s still agree with `ColumnElement`.
 * But the rendered SVG's `data-part-index` (what `usePlaybackCursor`'s DOM
 * lookups key off, see `usePlaybackCursor.ts`) is *compacted*: hiding a part
 * physically removes it before compiling (`apply_track_filter`), shifting
 * every later part's index down. Playing a selection that includes a part
 * declared after a hidden one queried the DOM for that part's true (higher)
 * index, which no longer exists there — so its notes' cursor highlight
 * never appeared at all.
 *
 * Fix: the worker now remaps each returned `NoteTiming.source_part_index`
 * from the true written index into the same hidden-parts-compacted space
 * the SVG uses, via `remapNoteTimingsToVisiblePartIndex` (see
 * `worker/audioMessageHandlers.ts`), using a `visibleTracks` field that
 * always carries the part-visibility toggle's own state — independent of
 * whatever narrower subset this particular clip further muted for playback.
 *
 * Fixture: three parts, Melody / Harmony / Bass. Harmony (the middle part)
 * is hidden, which compacts Bass from written part-index 2 down to rendered
 * part-index 1. Only Bass's two notes are drag-selected and played, so the
 * cursor highlight can only appear via the rendered (compacted) index —
 * never the stale, nonexistent written index 2.
 */
const source = [
  '# metadata',
  'title = "playback cursor selection hidden part test"',
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
        active: 'playback-cursor-hidden-part-test.jianpu',
        userFiles: { 'playback-cursor-hidden-part-test.jianpu': src },
        bin: {},
        fileIds: {
          'playback-cursor-hidden-part-test.jianpu':
            'playback-cursor-hidden-part-test-id-001',
        },
      }),
    )
  }, source)
}

test('playing a drag-selected part plays its notes and moves the playback cursor onto them, even when an earlier part is hidden', async ({
  page,
}) => {
  test.setTimeout(75_000)

  await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="measure"][data-measure-index="0"]', {
    timeout: 10_000,
  })

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
  // with the new enabledTracks before dragging.
  await page.waitForTimeout(2000)

  // Drag-select only Bass's two notes (rendered/compacted part-index 1),
  // never touching Melody's row above.
  const bassNotes = page.locator(
    '[data-tag="note"][data-part-index="1"] rect[data-variant="note-click-target-rect"]',
  )
  await expect(bassNotes).toHaveCount(2)
  const firstBox = await bassNotes.nth(0).boundingBox()
  const secondBox = await bassNotes.nth(1).boundingBox()
  if (!firstBox || !secondBox) {
    throw new Error("Could not get bounding boxes for Bass's notes.")
  }
  await page.mouse.move(firstBox.x + 2, firstBox.y + firstBox.height / 2)
  await page.mouse.down()
  await page.mouse.move(
    secondBox.x + secondBox.width - 2,
    secondBox.y + secondBox.height / 2,
    { steps: 10 },
  )
  await page.mouse.up()

  await expect(
    page.locator(
      '[data-tag="note"][data-note-drag-selected][data-part-index="1"]',
    ),
  ).toHaveCount(2)
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(2)

  const playBtn = page.locator('button.play-measure-btn')
  await expect(playBtn).toHaveText(/Selection/, { timeout: 5_000 })
  await expect(playBtn).toBeEnabled({ timeout: 30_000 })
  await playBtn.click()

  // Bass's first selected note must actually receive the playback cursor
  // highlight at the rendered (compacted) part-index 1 — before the fix,
  // the highlight was looked up at the stale written index 2, which no
  // rendered element has, so this never happened.
  await expect(
    page.locator(
      '[data-tag="note"][data-part-index="1"][data-note-id="0"] rect[data-variant="playback-cursor-rect"]',
    ),
  ).toHaveAttribute('fill', 'rgba(220,38,38,0.25)', { timeout: 20_000 })
})
