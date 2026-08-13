import { expect, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

/**
 * Selecting text directly in the Monaco editor (not dragging in the SVG
 * preview) highlights the corresponding notes in the preview — the reverse
 * direction of the note drag-select pathway. See
 * `useNoteSelection.ts`'s `handleEditorSelectionChange`, wired off
 * `Editor.tsx`'s `onSelectionOffsetChange` in `AppWorkspace.tsx`.
 *
 * Self-contained source (not a demo file) with a generous "max measures per
 * system" so all measures render in one row and stay within the viewport.
 *
 * Measure 0 : [M] 1 2 3 4   — 4 notes — line 9
 * Measure 1 : [M] 5 6       — 2 notes — line 11
 * Measure 2 : [M] 7 1'      — 2 notes — line 13
 */
const editorSelectionTestSource = [
  '# metadata',
  'title = "editor selection test"',
  'max_measures_per_system = 48',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '[M] 1 2 3 4', // measure 0 — line 9
  '',
  '[M] 5 6', // measure 1 — line 11
  '',
  "[M] 7 1'", // measure 2 — line 13
].join('\n')

async function loadFixture(page: import('@playwright/test').Page) {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'editor-selection-test.jianpu',
        userFiles: { 'editor-selection-test.jianpu': source },
        bin: {},
        fileIds: {
          'editor-selection-test.jianpu': 'editor-selection-test-id-001',
        },
      }),
    )
  }, editorSelectionTestSource)
}

test('selecting a measure line in the editor highlights its notes in the preview', async ({
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

  await focusEditor(page)
  // Jump to measure 1's line ("[M] 5 6") and select the whole line — a
  // plain text selection made in the editor itself, no preview interaction.
  await page.keyboard.press('Control+g')
  await page.keyboard.type('11')
  await page.keyboard.press('Enter')
  // Wait for the go-to-line jump to actually land (same priming dance the
  // click specs use — see `primeMeasureSpans`) before selecting, otherwise
  // `Home`/`Shift+End` can race the widget closing and land on the wrong
  // line or a stale cursor position.
  await expect(page.locator('button.play-measure-btn')).toHaveText(/Measure/, {
    timeout: 5_000,
  })
  await expect(
    page.locator('.preview-page [data-testid="measure-highlight"]').first(),
  ).toBeVisible({ timeout: 5_000 })
  await page.keyboard.press('Home')
  await page.keyboard.press('Shift+End')

  // Measure 1 ("5 6") has exactly 2 notes.
  const highlightedNotes = page.locator(
    '[data-tag="note"][data-note-drag-selected]',
  )
  await expect(highlightedNotes).toHaveCount(2)

  // The repurposed play-measure button switching to "▶ Selection" confirms
  // the editor selection was recognized as a real note range, same as a
  // preview-side drag.
  await expect(page.locator('button.play-measure-btn')).toHaveText(
    /Selection/,
    { timeout: 3_000 },
  )

  // Collapsing the selection back to a cursor clears the reflected
  // highlight — it only tracks an actual (non-empty) selection.
  await page.keyboard.press('Home')
  await expect(highlightedNotes).toHaveCount(0)
})
