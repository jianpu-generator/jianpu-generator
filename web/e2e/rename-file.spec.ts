import { expect, test } from '@playwright/test'

const SOURCE = [
  '# metadata',
  'title = "Rename Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

test('SVG preview persists after renaming the active file', async ({
  page,
}) => {
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

  // Wait for the initial SVG preview to appear.
  await page.waitForSelector('.preview-page', { timeout: 15_000 })

  // Double-click the active tab to enter rename mode.
  const tabName = page.locator('.file-tab--active .file-tab-name')
  await tabName.dblclick()

  // Clear and type a new name, then confirm.
  const input = page.locator('.file-tab--active input.file-tab-name')
  await input.fill('renamed.jianpu')
  await input.press('Enter')

  // The tab should show the new name.
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    'renamed.jianpu',
  )

  // The SVG preview should still be visible without any manual edits.
  await expect(page.locator('.preview-page').first()).toBeVisible({
    timeout: 5_000,
  })
})
