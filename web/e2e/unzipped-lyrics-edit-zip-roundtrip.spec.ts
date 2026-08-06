import { expect, type Page, test } from '@playwright/test'
import { focusEditor } from './fileSwitcherHelpers'

declare global {
  interface Window {
    monaco?: typeof import('monaco-editor')
  }
}

// Regression test: editing a lyrics verse in Unzipped view and switching
// back to Zipped view should reproduce exactly what was typed, not drop or
// otherwise mangle it.
const SOURCE = [
  '# parts',
  'a = notes+lyrics',
  '',
  '# score',
  '[a] 1 2 3 4',
  '[a] he llo world yes',
  '[a] ba ha ta',
].join('\n')

const EXPECTED_ZIPPED = [
  '# parts',
  'a = notes+lyrics',
  '',
  '# score',
  '[a] 1 2 3 4',
  '[a] he llo world yes',
  '[a] ba ha ta na',
  '',
].join('\n')

async function loadSource(page: Page) {
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
}

async function getEditorValue(page: Page) {
  return page.evaluate(
    () => window.monaco?.editor.getEditors()[0]?.getModel()?.getValue() ?? '',
  )
}

async function toggleUnzippedView(page: Page) {
  await page.locator('.pane-divider-view-toggle').click()
}

// Places the cursor right after `needle`'s last occurrence in the editor and
// focuses the editor, so a subsequent `page.keyboard.type` inserts there.
async function placeCursorAfter(page: Page, needle: string) {
  await page.evaluate((needle) => {
    const editor = window.monaco?.editor.getEditors()[0]
    const model = editor?.getModel()
    if (!editor || !model) throw new Error('Editor not ready')
    const text = model.getValue()
    const index = text.lastIndexOf(needle)
    if (index === -1) throw new Error(`"${needle}" not found in editor text`)
    const pos = model.getPositionAt(index + needle.length)
    editor.focus()
    editor.setPosition(pos)
    editor.revealPositionInCenter(pos)
  }, needle)
}

test.beforeEach(async ({ page }) => {
  await loadSource(page)
})

test('appending a syllable to a lyrics verse in Unzipped view survives switching back to Zipped view', async ({
  page,
}) => {
  await focusEditor(page)

  await toggleUnzippedView(page)
  await expect
    .poll(async () => (await getEditorValue(page)).replace(/\r\n/g, '\n'))
    .toContain('[a:lyrics:2]\nba ha ta')

  // Append " na" to the end of verse 2's "ba ha ta", making it "ba ha ta na".
  await placeCursorAfter(page, 'ba ha ta')
  await page.keyboard.type(' na')

  await expect
    .poll(async () => (await getEditorValue(page)).replace(/\r\n/g, '\n'))
    .toContain('ba ha ta na')

  await toggleUnzippedView(page)

  await expect
    .poll(async () => (await getEditorValue(page)).replace(/\r\n/g, '\n'))
    .toBe(EXPECTED_ZIPPED)
})
