import { expect, test } from '@playwright/test'
import { fileSwitcherTrigger, openFileList } from './fileSwitcherHelpers'

const SOURCE = [
  '# metadata',
  'title = "Extension Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

test('file picker hides the redundant .jianpu extension', async ({ page }) => {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'my song.jianpu',
        userFiles: { 'my song.jianpu': src },
        bin: {},
        fileIds: { 'my song.jianpu': crypto.randomUUID() },
      }),
    )
  }, SOURCE)

  await page.goto('/')
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  // The trigger button shows the active file's name without ".jianpu".
  await expect(fileSwitcherTrigger(page)).toHaveText('my song')

  // The file list's tab entry also omits the extension.
  await openFileList(page)
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    'my song',
  )

  // Double-clicking to rename starts from the extension-less name, and
  // typing a bare name (no ".jianpu") still produces a valid rename — the
  // extension is re-added under the hood.
  const tabName = page.locator('.file-tab--active .file-tab-name')
  await tabName.dblclick()
  const input = page.locator('.file-tab--active input.file-tab-name')
  await expect(input).toHaveValue('my song')
  await input.fill('renamed')
  await input.press('Enter')

  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    'renamed',
  )
  await expect(fileSwitcherTrigger(page)).toHaveText('renamed')
})
