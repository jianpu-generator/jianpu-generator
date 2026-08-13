import { expect, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

/**
 * Regression test for the note drag-select highlight vanishing right after
 * mouseup (see `Preview.tsx`'s `applyPersistedNoteHighlights` /
 * `selectedNoteCells` prop): the highlight used to be a one-shot imperative
 * DOM toggle that got explicitly cleared on mouseup, and — even without that
 * bug — would still have been wiped by the highlighted-SVG re-render that
 * the drag's own Monaco selection triggers a moment later.
 *
 * Self-contained source (not a demo file) with a generous "max measures per
 * system" and four single-beat notes in one measure, so all four note
 * click-targets render side by side in one row and stay within the viewport
 * during the drag.
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
  '[M] 1 2 3 4', // measure 0 — line 9
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

test('note drag-select highlight stays visible after mouseup and the subsequent Monaco-triggered re-render', async ({
  page,
}) => {
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

  // Prime the editor/worker round-trip the same way the measure drag-select
  // spec does, so the highlighted-documents re-render this test is guarding
  // against is actually wired up before we drag.
  await focusEditor(page)
  await page.keyboard.press('Control+g')
  await page.keyboard.type('9')
  await page.keyboard.press('Enter')
  await expect(page.locator('button.play-measure-btn')).toHaveText(/Measure/, {
    timeout: 5_000,
  })

  const box0 = await noteRects.nth(0).boundingBox()
  const box2 = await noteRects.nth(2).boundingBox()
  if (!box0 || !box2) {
    throw new Error(
      'Could not get bounding boxes for notes 0 and 2. ' +
        'Ensure the SVG preview has rendered.',
    )
  }

  const startX = box0.x + box0.width / 2
  const startY = box0.y + box0.height / 2
  const endX = box2.x + box2.width / 2
  const endY = box2.y + box2.height / 2

  // Drag a marquee across the first three notes.
  await page.mouse.move(startX, startY)
  await page.mouse.down()
  await page.mouse.move(endX, endY, { steps: 10 })
  await page.mouse.up()

  const highlightedNotes = page.locator(
    '[data-tag="note"][data-note-drag-selected]',
  )

  // Immediately after mouseup, the highlight must still be showing (this is
  // the bug: it used to be cleared the instant mouseup ran).
  await expect(highlightedNotes).toHaveCount(3)

  // The repurposed play-measure button switching to "▶ Selection" confirms a
  // note range was pushed into Monaco/App state.
  await expect(page.locator('button.play-measure-btn')).toHaveText(
    /Selection/,
    { timeout: 3_000 },
  )

  // Dragging a note-range selection pushes a Monaco multicursor selection,
  // whose cursor-change listener debounces (300 ms) into a worker
  // round-trip that swaps the plain SVG documents for highlighted ones —
  // wiping any highlight applied only as a one-shot DOM mutation. The
  // highlight must survive that swap too.
  await page.waitForTimeout(700)
  await expect(highlightedNotes).toHaveCount(3)
})
