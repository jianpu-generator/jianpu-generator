import { expect, test } from '@playwright/test'

/**
 * Regression test: a `# sequence` chain selection whose entries are
 * disjoint in the source (here dragging "C" to a later repeat of "A" across
 * "A, B, C, A") must highlight exactly those entries' own measures in the
 * SVG preview — not the measure sitting between them ("B") — mirroring the
 * disjoint Monaco selection `sequence-jump-select-out-of-order.spec.ts`
 * already covers for the editor side.
 *
 * Lines (1-based):
 *    7: # sequence
 *    8: A, B, C, A
 *   11: time=4/4 key=C4 bpm=120 label="A"  ← view-zone directive
 *   12: 1 2 3 4                             ← measure 0 ("A")
 *   14: label="B"                           ← view-zone directive
 *   15: 5 6 7 1'                            ← measure 1 ("B")
 *   17: label="C"                           ← view-zone directive
 *   18: 1' 7 6 5                            ← measure 2 ("C")
 *
 * The sequence's second "A" is a repeat — it reuses measure 0's written
 * lines rather than duplicating them, so the document has only 3 written
 * measures for 4 sequence entries.
 */
const source = [
  '# metadata',
  'title = "test"',
  '',
  '# parts',
  'M = notes',
  '',
  '# sequence',
  'A, B, C, A',
  '',
  '# score',
  'time=4/4 key=C4 bpm=120 label="A"',
  '1 2 3 4',
  '',
  'label="B"',
  "5 6 7 1'",
  '',
  'label="C"',
  "1' 7 6 5",
].join('\n')

async function getEditorSelections(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const monacoApi = (
      window as unknown as { monaco: typeof import('monaco-editor') }
    ).monaco
    const selections = monacoApi.editor.getEditors()[0]?.getSelections() ?? []
    return selections.map((s) => ({
      startLineNumber: s.startLineNumber,
      endLineNumber: s.endLineNumber,
    }))
  })
}

// Both toolbars render `button.section-jump-btn` inside their own
// `[role="toolbar"]`; SequenceJumpToolbar mounts second (see
// sequence-jump-select.spec.ts).
function sequenceToolbarButtons(page: import('@playwright/test').Page) {
  return page
    .locator('[role="toolbar"]')
    .nth(1)
    .locator('button.section-jump-btn')
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'sequence-chain-highlight-test.jianpu',
        userFiles: { 'sequence-chain-highlight-test.jianpu': src },
        bin: {},
        fileIds: {
          'sequence-chain-highlight-test.jianpu': crypto.randomUUID(),
        },
      }),
    )
  }, source)

  await page.goto('/')
  // Four sequence entries: "A", "B", "C", "A" (the repeat).
  await expect(sequenceToolbarButtons(page)).toHaveCount(4, {
    timeout: 15_000,
  })
})

test('dragging "C" to the repeated "A" selects both entries in the editor and highlights both measures in the preview, excluding "B"', async ({
  page,
}) => {
  const buttons = sequenceToolbarButtons(page)

  // Drag from the "C" button (index 2) to the second "A" button (index 3,
  // the repeat).
  await buttons.nth(2).hover()
  await page.mouse.down()
  await buttons.nth(3).hover()
  await page.mouse.up()

  await expect
    .poll(() => getEditorSelections(page), { timeout: 3_000 })
    .toEqual([
      { startLineNumber: 17, endLineNumber: 18 }, // "C" only
      { startLineNumber: 11, endLineNumber: 12 }, // "A" only (the repeat resolves to the same written lines)
    ])

  const highlightRects = page.locator(
    '.preview-page [data-testid="measure-highlight"]',
  )
  await expect(highlightRects).toHaveCount(2, { timeout: 3_000 })

  const measureBox = async (measureIndex: number) =>
    page
      .locator(`[data-tag="measure"][data-measure-index="${measureIndex}"]`)
      .first()
      .boundingBox()

  const cBox = await measureBox(2)
  const aBox = await measureBox(0)
  const bBox = await measureBox(1)
  if (!cBox || !aBox || !bBox) {
    throw new Error('Could not get bounding boxes for the measures.')
  }

  const highlightBoxes = await highlightRects.evaluateAll((rects) =>
    rects.map((r) => r.getBoundingClientRect()),
  )

  const matchesBox = (
    highlight: { x: number; y: number },
    target: { x: number; y: number },
  ) =>
    Math.abs(highlight.x - target.x) < 5 && Math.abs(highlight.y - target.y) < 5

  expect(highlightBoxes.some((h) => matchesBox(h, cBox))).toBe(true)
  expect(highlightBoxes.some((h) => matchesBox(h, aBox))).toBe(true)
  expect(highlightBoxes.some((h) => matchesBox(h, bBox))).toBe(false)
})
