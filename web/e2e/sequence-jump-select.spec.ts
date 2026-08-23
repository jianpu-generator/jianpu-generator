import { expect, test } from '@playwright/test'

/**
 * Regression test for PENDING_TASK_sequence-entries-no-highlight.md:
 * clicking a "sequence entries" toolbar button updated only local button
 * state (`useSequenceNavigation`'s `selectedIndexRange`) — it never touched
 * the Monaco selection or the SVG preview, unlike its sibling Section
 * toolbar. `useSequenceNavigation` now also takes `measureSpans`,
 * `editorRef`, and `notifySelection` and mirrors
 * `useSectionNavigation`'s `selectSectionRange`.
 *
 * Uses the same two-section (A, B) source as section-jump-select.spec.ts,
 * plus a passthrough `# sequence` (`A, B`) so the resolved sequence entries
 * map 1:1 onto the same measure ranges as the section buttons — letting
 * this test assert the exact same line numbers that test already
 * establishes for the Section toolbar.
 *
 * Lines (1-based):
 *   11: time=4/4 key=C4 bpm=120 label="A"       ← view-zone directive
 *   12: 1 2 3 4                                   ← measure 0
 *   13: (blank)
 *   14: 5 6 7 1'                                 ← measure 1
 *   15: (blank)
 *   16: label="B"                                ← view-zone directive
 *   17: 1' 7 6 5                                 ← measure 2
 *   18: (blank)
 *   19: 4 3 2 1                                  ← measure 3
 */
const source = [
  '# metadata',
  'title = "test"',
  '',
  '# parts',
  'M = notes',
  '',
  '# sequence',
  'A, B',
  '',
  '# score',
  'time=4/4 key=C4 bpm=120 label="A"',
  '1 2 3 4',
  '',
  "5 6 7 1'",
  '',
  'label="B"',
  "1' 7 6 5",
  '',
  '4 3 2 1',
].join('\n')

async function getEditorSelection(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const monacoApi = (
      window as unknown as { monaco: typeof import('monaco-editor') }
    ).monaco
    const selection = monacoApi.editor.getEditors()[0]?.getSelection()
    if (!selection) return null
    return {
      startLineNumber: selection.startLineNumber,
      endLineNumber: selection.endLineNumber,
    }
  })
}

// Both toolbars render `button.section-jump-btn` inside their own
// `[role="toolbar"]`; SequenceJumpToolbar mounts after SectionJumpToolbar in
// App.tsx, so it's the second one (see sequence-jump-toolbar.spec.ts).
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
        active: 'sequence-select-test.jianpu',
        userFiles: { 'sequence-select-test.jianpu': src },
        bin: {},
        fileIds: { 'sequence-select-test.jianpu': crypto.randomUUID() },
      }),
    )
  }, source)

  await page.goto('/')
  await expect(sequenceToolbarButtons(page)).toHaveCount(2, {
    timeout: 15_000,
  })
})

test('clicking the "A" sequence entry selects measures 0–1 and highlights lines 11–14 in Monaco', async ({
  page,
}) => {
  await sequenceToolbarButtons(page).nth(0).click()

  await expect(page.getByTestId('selected-measure-range')).toHaveText('0-1', {
    timeout: 3_000,
  })
  await expect
    .poll(() => getEditorSelection(page), { timeout: 3_000 })
    .toEqual({ startLineNumber: 11, endLineNumber: 14 })
})

test('clicking the "B" sequence entry selects measures 2–3 and highlights lines 16–19 in Monaco', async ({
  page,
}) => {
  await sequenceToolbarButtons(page).nth(1).click()

  await expect(page.getByTestId('selected-measure-range')).toHaveText('2-3', {
    timeout: 3_000,
  })
  await expect
    .poll(() => getEditorSelection(page), { timeout: 3_000 })
    .toEqual({ startLineNumber: 16, endLineNumber: 19 })
})
