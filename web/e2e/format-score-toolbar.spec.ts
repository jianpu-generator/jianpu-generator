import { expect, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

declare global {
  interface Window {
    monaco?: typeof import('monaco-editor')
  }
}

// Bass has real content in both measure groups, so the trailing all-rest
// Melody line in the second group is safe to drop without emptying the
// group. `[Bass]   5  6 7 1` carries irregular whitespace to exercise the
// collapse-to-single-space cleanup at the same time.
const SOURCE = [
  '# parts',
  'Melody = notes',
  'Bass = notes',
  '',
  '# score',
  '[Melody] 1 2 3 4',
  '[Bass]   5  6 7 1',
  '',
  '[Melody] 0 0 0 0',
  '[Bass] 3 3 3 3',
].join('\n')

const FORMATTED = [
  '# parts',
  'Melody = notes',
  'Bass = notes',
  '',
  '# score',
  '[Melody] 1 2 3 4',
  '[Bass] 5 6 7 1',
  '',
  '[Bass] 3 3 3 3',
  '',
].join('\n')

test.beforeEach(async ({ page }) => {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'original.jianpu',
        userFiles: { 'original.jianpu': src },
        bin: {},
        fileIds: { 'original.jianpu': crypto.randomUUID() },
      }),
    )
  }, SOURCE)

  await page.goto('/')
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

async function getEditorValue(page: import('@playwright/test').Page) {
  return page.evaluate(
    () => window.monaco?.editor.getEditors()[0]?.getModel()?.getValue() ?? '',
  )
}

async function getEditorPosition(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const pos = window.monaco?.editor.getEditors()[0]?.getPosition()
    return pos ? { lineNumber: pos.lineNumber, column: pos.column } : null
  })
}

async function setEditorPosition(
  page: import('@playwright/test').Page,
  lineNumber: number,
  column: number,
) {
  await page.evaluate(
    ({ lineNumber, column }) => {
      window.monaco?.editor.getEditors()[0]?.setPosition({
        lineNumber,
        column,
      })
    },
    { lineNumber, column },
  )
}

test('drops redundant lines and collapses whitespace, keeping the caret in place when its position is still valid', async ({
  page,
}) => {
  await focusEditor(page)
  // Column 5 lands inside "[Mel" on the first score line, whose content and
  // line number are both unaffected by formatting.
  await setEditorPosition(page, 6, 5)

  await page.click('.pane-divider-format-toggle')

  await expect.poll(() => getEditorValue(page)).toBe(FORMATTED)
  await expect
    .poll(() => getEditorPosition(page))
    .toEqual({
      lineNumber: 6,
      column: 5,
    })
})

test('clamps the caret to a valid position when its line is dropped by formatting', async ({
  page,
}) => {
  await focusEditor(page)
  // Line 9 is "[Melody] 0 0 0 0" (17 chars), and column 18 is its end —
  // that whole line is dropped by formatting, so this position can't
  // survive as-is.
  await setEditorPosition(page, 9, 18)

  await page.click('.pane-divider-format-toggle')

  await expect.poll(() => getEditorValue(page)).toBe(FORMATTED)

  const formattedLines = FORMATTED.split('\n')
  await expect
    .poll(async () => {
      const pos = await getEditorPosition(page)
      if (!pos) return null
      const lineContent = formattedLines[pos.lineNumber - 1]
      if (lineContent === undefined) return null
      return pos.column <= lineContent.length + 1
    })
    .toBe(true)
})
