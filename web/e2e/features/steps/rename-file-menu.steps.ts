import { expect } from '@playwright/test'
import { fileSwitcherTrigger, openFileActions } from '../../fileSwitcherHelpers'
import { Given, Then, When } from './fixtures'

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

Given(
  'a local-storage-backed file {string} is seeded for the rename menu test',
  async ({ page }, name: string) => {
    await page.addInitScript(
      ({ src, fileName }) => {
        localStorage.setItem(
          'jianpu:files:v1',
          JSON.stringify({
            active: fileName,
            userFiles: { [fileName]: src },
            bin: {},
            fileIds: { [fileName]: crypto.randomUUID() },
          }),
        )
      },
      { src: SOURCE, fileName: name },
    )
  },
)

Given('the app loads the seeded rename-menu test file', async ({ page }) => {
  await page.goto('/')
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

Given(
  'the rename dialog will be accepted with {string}',
  async ({ page }, name: string) => {
    page.once('dialog', (dialog) => {
      void dialog.accept(name)
    })
  },
)

Given('the rename dialog will be dismissed', async ({ page }) => {
  page.once('dialog', (dialog) => {
    void dialog.dismiss()
  })
})

When('I open file actions and click the Rename menu item', async ({ page }) => {
  await openFileActions(page)
  await page.getByRole('menuitem', { name: 'Rename' }).click()
})

Then(
  'the file switcher trigger shows {string} after the rename prompt',
  async ({ page }, name: string) => {
    await expect(fileSwitcherTrigger(page)).toContainText(name)
  },
)
