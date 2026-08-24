import { expect, test } from '@playwright/test'

/**
 * Regression test: a `# sequence` chain referencing sections out of
 * document order (here `C, A`, against sections declared `A, B, C`) must
 * select exactly the chain's own entries as disjoint Monaco ranges — not
 * collapse into one contiguous range that also sweeps up whatever sits
 * between them in the source (`B`), and not drop either endpoint entirely.
 *
 * Lines (1-based):
 *    7: # sequence
 *    8: C, A
 *   11: time=4/4 key=C4 bpm=120 label="A"  ← view-zone directive
 *   12: 1 2 3 4                             ← measure 0 ("A")
 *   14: label="B"                           ← view-zone directive
 *   15: 5 6 7 1'                            ← measure 1 ("B")
 *   17: label="C"                           ← view-zone directive
 *   18: 1' 7 6 5                            ← measure 2 ("C")
 */
const source = [
  '# metadata',
  'title = "test"',
  '',
  '# parts',
  'M = notes',
  '',
  '# sequence',
  'C, A',
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
        active: 'sequence-out-of-order-test.jianpu',
        userFiles: { 'sequence-out-of-order-test.jianpu': src },
        bin: {},
        fileIds: { 'sequence-out-of-order-test.jianpu': crypto.randomUUID() },
      }),
    )
  }, source)

  await page.goto('/')
  await expect(sequenceToolbarButtons(page)).toHaveCount(2, {
    timeout: 15_000,
  })
})

test('drag-selecting the out-of-order "C, A" chain selects both entries as disjoint ranges, excluding "B"', async ({
  page,
}) => {
  const buttons = sequenceToolbarButtons(page)

  // Drag from the "C" button (index 0, chain-first) to the "A" button
  // (index 1, chain-last) — chain order disagrees with document order here.
  await buttons.nth(0).hover()
  await page.mouse.down()
  await buttons.nth(1).hover()
  await page.mouse.up()

  await expect(page.getByTestId('selected-measure-range')).toHaveText('0-2', {
    timeout: 3_000,
  })

  await expect
    .poll(() => getEditorSelections(page), { timeout: 3_000 })
    .toEqual([
      { startLineNumber: 17, endLineNumber: 18 }, // "C" only
      { startLineNumber: 11, endLineNumber: 12 }, // "A" only
    ])
})
