import { expect, test } from '@playwright/test'
import { fileSwitcherTrigger, openFileActions } from './fileSwitcherHelpers'

const SOURCE = [
  '# metadata',
  'title = "Rename Menu Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
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

test('renaming via the "⋯" menu prompt updates the active tab and trigger', async ({
  page,
}) => {
  page.once('dialog', (dialog) => {
    void dialog.accept('renamed')
  })

  await openFileActions(page)
  await page.getByRole('menuitem', { name: 'Rename' }).click()

  await expect(fileSwitcherTrigger(page)).toContainText('renamed')
})

test('cancelling the rename prompt leaves the filename unchanged', async ({
  page,
}) => {
  page.once('dialog', (dialog) => {
    void dialog.dismiss()
  })

  await openFileActions(page)
  await page.getByRole('menuitem', { name: 'Rename' }).click()

  await expect(fileSwitcherTrigger(page)).toContainText('original')
})
