import { expect, test } from '@playwright/test'

/**
 * Regression test: a measure/bar-line drag that spans multiple systems used
 * to mis-select notes belonging to a part declared *after* a
 * currently-hidden part.
 *
 * Root cause: `noteSpans` (fetched via the `listNoteSpans` worker message,
 * see `jianpu.worker.ts`) comes from `list_note_spans_from_source`, which
 * used to compile the score with no track filter at all — so each span's
 * `sourcePartIndex` was the event's raw index into `MultiPartMeasure::parts`,
 * i.e. every declared part counted, including hidden ones.
 *
 * The rendered SVG is different: `render_svgs_with_parts`'s pipeline runs
 * `apply_track_filter` first (see `document_render.rs`/`filters.rs`), which
 * `Vec::retain`s hidden parts *out of* `measure.parts` before compiling —
 * so every part declared after a hidden one is compacted down by one index
 * in the SVG's `data-part-index` attributes.
 *
 * `usePreviewDragSelection.ts`'s 'measure' mode (`noteCellsInMeasureRange`)
 * resolves a drag's selected cells straight from `noteSpans` and then marks
 * the matching `data-part-index`/`data-note-id` DOM groups
 * (`applyPersistedNoteHighlights`) — so once those two "part index"
 * numberings disagreed, cells resolved from `noteSpans` no longer lined up
 * with the compacted indices actually present in the DOM for any part
 * declared after the hidden one.
 *
 * Fix: `list_note_spans_from_source`/`list_lyric_spans_from_source` (and
 * their wasm bindings `list_note_spans`/`list_lyric_spans`) now take an
 * `enabled_tracks` filter and apply `apply_track_filter` themselves before
 * walking the score, so their indices always match whatever the renderer
 * used for the same `enabledTracks`. The frontend now threads the current
 * `enabledTracks` through the `listNoteSpans`/`listLyricSpans` worker
 * messages (see `useJianpuWorkerRenderRequests.ts`) instead of omitting it.
 *
 * Fixture: three parts, Melody / Harmony / Bass, two systems (one measure
 * each). Harmony (the middle part) is hidden, which compacts Bass from
 * source part-index 2 down to rendered part-index 1. Harmony has 1 note per
 * measure and Bass has 2, so — before the fix — Harmony's unfiltered
 * `sourcePartIndex:noteId` keys ("1:0") only ever accidentally collided
 * with *some* of Bass's rendered keys: enough to make system 0 (measure 0)
 * look right by coincidence, while system 1 (measure 1) had no such
 * collision and simply failed to select Bass's notes at all. That's the
 * "cross-system" symptom this guards against: the same drag behaving
 * correctly in the system it started in and silently dropping notes in the
 * next one.
 */
const source = [
  '# metadata',
  'title = "measure drag hidden part index mismatch"',
  'max_measures_per_system = 1',
  '',
  '# parts',
  'Melody [M] = notes',
  'Harmony [H] = notes',
  'Bass [B] = notes',
  '',
  '# score',
  '[M] 1 2', // measure 0 — system 0
  '[H] 5', // 1 note/measure
  '[B] 3 4', // 2 notes/measure
  '',
  '[M] 3 4', // measure 1 — system 1
  '[H] 7',
  '[B] 5 6',
].join('\n')

async function loadFixture(page: import('@playwright/test').Page) {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'measure-drag-hidden-part-test.jianpu',
        userFiles: { 'measure-drag-hidden-part-test.jianpu': src },
        bin: {},
        fileIds: {
          'measure-drag-hidden-part-test.jianpu':
            'measure-drag-hidden-part-test-id-001',
        },
      }),
    )
  }, source)
}

test('a measure drag across systems selects every visible part’s notes even when another part is hidden', async ({
  page,
}) => {
  await loadFixture(page)
  await page.goto('/')

  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await page.waitForSelector('[data-tag="measure"][data-measure-index="1"]', {
    timeout: 10_000,
  })

  // Hide Harmony — the middle part — so Bass compacts from rendered
  // part-index 2 down to 1.
  const harmonyPill = page.locator('.part-toggle-pill').filter({
    has: page.locator('.part-toggle-abbr', { hasText: /^H$/ }),
  })
  await harmonyPill.locator('.part-toggle-segment--eye').click()

  const noteRects = page.locator('rect[data-variant="note-click-target-rect"]')
  // Melody (2/measure) + Bass (2/measure), 2 measures = 8 rendered notes.
  await expect(noteRects).toHaveCount(8, { timeout: 10_000 })
  // Give the debounced listNoteSpans worker round-trip time to catch up
  // with the new enabledTracks before dragging.
  await page.waitForTimeout(2000)

  const measures = page.locator('[data-tag="measure"]')
  const firstBox = await measures.nth(0).boundingBox()
  const lastBox = await measures.nth((await measures.count()) - 1).boundingBox()
  if (!firstBox || !lastBox) {
    throw new Error('Could not get bounding boxes for measures 0 and 1.')
  }

  // Start the drag exactly on measure 0's left bar line and drag into
  // measure 1's interior — a measure-mode drag spanning both systems.
  await page.mouse.move(firstBox.x, firstBox.y + firstBox.height / 2)
  await page.mouse.down()
  await page.mouse.move(
    lastBox.x + lastBox.width / 2,
    lastBox.y + lastBox.height / 2,
    { steps: 10 },
  )
  await page.mouse.up()

  // Every rendered note (Melody's 4 + Bass's 4) should be selected — Bass
  // is a fully visible part and both its measures sit inside the dragged
  // range.
  await expect(
    page.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(8)

  // In particular, Bass's system-1 (measure 1) notes — note ids 2 and 3,
  // since each part's note-id counter runs across the whole score rather
  // than resetting per measure — must be selected. This is the pair the
  // index mismatch silently drops (Harmony's unfiltered spans only ever
  // collide with Bass's measure-0 ids 0/1, not measure-1's 2/3).
  await expect(
    page.locator(
      '[data-tag="note"][data-note-drag-selected][data-part-index="1"][data-note-id="2"]',
    ),
  ).toHaveCount(1)
  await expect(
    page.locator(
      '[data-tag="note"][data-note-drag-selected][data-part-index="1"][data-note-id="3"]',
    ),
  ).toHaveCount(1)
})
