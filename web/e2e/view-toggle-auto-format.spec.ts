import { expect, type Page, test } from '@playwright/test'
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

const FORMATTED_ZIPPED = [
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

test.beforeEach(async ({ page }) => {
  await loadSource(page)
})

test('switching to Unzipped view auto-applies the Unzipped formatter, breaking each measure onto its own line', async ({
  page,
}) => {
  await focusEditor(page)

  await toggleUnzippedView(page)

  await expect
    .poll(async () => (await getEditorValue(page)).replace(/\r\n/g, '\n'))
    .toContain('[Melody]\n1 2 3 4\n0 0 0 0')
  await expect
    .poll(async () => (await getEditorValue(page)).replace(/\r\n/g, '\n'))
    .toContain('[Bass]\n5 6 7 1\n3 3 3 3')
})

test('switching back to Zipped view auto-applies the Zipped formatter, matching the manual Format action', async ({
  page,
}) => {
  await focusEditor(page)

  await toggleUnzippedView(page)
  await expect.poll(() => getEditorValue(page)).toContain('[Melody]')

  await toggleUnzippedView(page)

  await expect
    .poll(async () => (await getEditorValue(page)).replace(/\r\n/g, '\n'))
    .toBe(FORMATTED_ZIPPED)
})
