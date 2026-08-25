import { expect } from '@playwright/test'
import { fileSwitcherTrigger, openFileActions } from '../../fileSwitcherHelpers'
import { encodeShareHashOnPage, gotoShareUrl } from '../../shareUrlHelper'
import { Given, Then, When } from './fixtures'

const FILE_STORE_KEY = 'jianpu:files:v1'
const SHARED_FILENAME = 'shared-test.jianpu'
const SHARED_SOURCE = [
  '# metadata',
  'title = "Shared Score"',
  '',
  '# parts',
  'Melody = notes',
  '',
  '# score',
  '(time=4/4 key=C4 bpm=120)',
  '1 2 3 4',
].join('\n')

let lastShareUrl: string | undefined

Given('local storage is cleared, as seen in share', async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.clear()
  })
})

When(
  'I open the share URL for {string}',
  async ({ page }, filename: string) => {
    expect(filename).toBe(SHARED_FILENAME)
    await gotoShareUrl(page, SHARED_FILENAME, SHARED_SOURCE)
  },
)

When(
  'I open the share URL for {string} again',
  async ({ page }, filename: string) => {
    expect(filename).toBe(SHARED_FILENAME)
    await gotoShareUrl(page, SHARED_FILENAME, SHARED_SOURCE)
  },
)

When('I reload the page', async ({ page }) => {
  await page.reload()
})

When(
  'I navigate to a legacy uncompressed share link for {string}',
  async ({ page }, filename: string) => {
    expect(filename).toBe(SHARED_FILENAME)
    const legacyPayload = encodeURIComponent(
      JSON.stringify({ filename: SHARED_FILENAME, content: SHARED_SOURCE }),
    )
    await page.goto(`http://localhost:5173/#share=${legacyPayload}`)
  },
)

When('I click {string}', async ({ page }, buttonName: string) => {
  await page.getByRole('button', { name: buttonName }).click()
})

When('I click the pane-divider toggle, as seen in share', async ({ page }) => {
  await page.locator('.pane-divider-toggle').click()
})

Given(
  'clipboard permissions are granted, as seen in share',
  async ({ context }) => {
    await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  },
)

Given(
  'a user file {string} is seeded in local storage',
  async ({ page }, filename: string) => {
    expect(filename).toBe(SHARED_FILENAME)
    await page.addInitScript(
      ({
        key,
        filename,
        source,
      }: {
        key: string
        filename: string
        source: string
      }) => {
        localStorage.setItem(
          key,
          JSON.stringify({
            active: filename,
            userFiles: { [filename]: source },
            bin: {},
            fileIds: { [filename]: 'share-test-id' },
          }),
        )
      },
      { key: FILE_STORE_KEY, filename: SHARED_FILENAME, source: SHARED_SOURCE },
    )
  },
)

When('the app loads, as seen in share', async ({ page }) => {
  await page.goto('/')
})

When(
  'I open the file actions menu and click the share button',
  async ({ page }) => {
    await openFileActions(page)
    await page.getByTestId('share-button').click()
  },
)

When('I navigate fresh to the copied share URL', async ({ page }) => {
  if (!lastShareUrl) {
    throw new Error('lastShareUrl was not captured before this step')
  }
  await page.goto('/')
  await page.evaluate(() => localStorage.clear())
  // A hash-only change from the current document is a same-document
  // navigation and won't remount the app, unlike a real recipient opening
  // the link fresh — force a full navigation via a blank interstitial page.
  await page.goto('about:blank')
  await page.goto(lastShareUrl)
})

Then(
  'the shared preview banner shows {string}',
  async ({ page }, filename: string) => {
    await expect(page.locator('.shared-preview-banner')).toContainText(filename)
  },
)

Then('the shared preview banner is visible', async ({ page }) => {
  await expect(page.locator('.shared-preview-banner')).toBeVisible()
})

Then('the shared preview banner is gone', async ({ page }) => {
  await expect(page.locator('.shared-preview-banner')).toHaveCount(0)
})

Then('the file switcher is hidden entirely', async ({ page }) => {
  // The file switcher is hidden entirely while previewing a shared score.
  await expect(fileSwitcherTrigger(page)).toHaveCount(0)
})

Then('the preview contains {string}', async ({ page }, text: string) => {
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  const previewContent = await page.locator('.preview-page').first().innerHTML()
  expect(previewContent).toContain(text)
})

Then('the file switcher shows {string}', async ({ page }, filename: string) => {
  await expect(fileSwitcherTrigger(page)).toContainText(filename)
})

Then(
  'the file switcher no longer shows {string}',
  async ({ page }, filename: string) => {
    await expect(fileSwitcherTrigger(page)).not.toContainText(filename)
  },
)

Then('the editor pane is collapsed, as seen in share', async ({ page }) => {
  await expect(page.locator('.pane--editor')).toHaveClass(
    /pane--editor-collapsed/,
  )
})

Then('the editor pane is expanded, as seen in share', async ({ page }) => {
  await expect(page.locator('.pane--editor')).not.toHaveClass(
    /pane--editor-collapsed/,
  )
})

Then('the pane-divider toggle is hidden', async ({ page }) => {
  await expect(page.locator('.pane-divider-toggle')).toHaveCount(0)
})

Then('the pane-divider toggle is visible again', async ({ page }) => {
  // The toggle reappears once the shared preview is dismissed, letting the
  // user re-expand the editor pane manually.
  await expect(page.locator('.pane-divider-toggle')).toBeVisible()
})

Then('the share button shows {string}', async ({ page }, text: string) => {
  await expect(page.getByTestId('share-button')).toHaveText(text)
})

Then(
  'the copied share URL matches the expected compressed hash for {string}',
  async ({ page }, filename: string) => {
    expect(filename).toBe(SHARED_FILENAME)
    const shareUrl = await page.evaluate(async () => {
      return navigator.clipboard.readText()
    })

    const expectedHash = await encodeShareHashOnPage(
      page,
      SHARED_FILENAME,
      SHARED_SOURCE,
    )
    expect(shareUrl).toContain(`#share=${expectedHash}`)
    lastShareUrl = shareUrl
  },
)
