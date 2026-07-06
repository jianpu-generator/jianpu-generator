import { expect, test } from '@playwright/test'
import { encodeShareHashSuffix } from '../src/shareUrl'

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

function shareUrlForLocalhost(filename: string, content: string): string {
  return `http://localhost:5173/#share=${encodeShareHashSuffix(filename, content)}`
}

test('opens a shared score preview without saving it, then imports on demand', async ({
  page,
}) => {
  await page.addInitScript(() => {
    localStorage.clear()
  })

  await page.goto(shareUrlForLocalhost(SHARED_FILENAME, SHARED_SOURCE))

  await expect(page.locator('.shared-preview-banner')).toContainText(
    SHARED_FILENAME,
  )
  await expect(
    page.locator('.file-tab--active .file-tab-name'),
  ).not.toHaveText(SHARED_FILENAME)

  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  const previewContent = await page.locator('.preview-page').first().innerHTML()
  expect(previewContent).toContain('Shared Score')

  // Reloading without importing must not have persisted the shared score.
  await page.reload()
  await expect(
    page.locator('.file-tab--active .file-tab-name'),
  ).not.toHaveText(SHARED_FILENAME)

  await page.goto(shareUrlForLocalhost(SHARED_FILENAME, SHARED_SOURCE))
  await page.getByRole('button', { name: 'Import to my scores' }).click()

  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    SHARED_FILENAME,
  )
  await expect(page.locator('.shared-preview-banner')).toHaveCount(0)
})

test('discarding a shared preview does not save it', async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.clear()
  })

  await page.goto(shareUrlForLocalhost(SHARED_FILENAME, SHARED_SOURCE))
  await expect(page.locator('.shared-preview-banner')).toBeVisible()

  await page.getByRole('button', { name: 'Discard' }).click()

  await expect(page.locator('.shared-preview-banner')).toHaveCount(0)
  await expect(
    page.locator('.file-tab--active .file-tab-name'),
  ).not.toHaveText(SHARED_FILENAME)
})

test('opens legacy uncompressed share links', async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.clear()
  })

  const legacyPayload = encodeURIComponent(
    JSON.stringify({ filename: SHARED_FILENAME, content: SHARED_SOURCE }),
  )
  await page.goto(`http://localhost:5173/#share=${legacyPayload}`)

  await expect(page.locator('.shared-preview-banner')).toContainText(
    SHARED_FILENAME,
  )
})

test('share button copies a compressed link that opens as a preview', async ({
  page,
  context,
}) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  await page.goto('/')
  await page.evaluate(
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
  await page.reload()

  await page.getByTestId('share-button').click()
  await expect(page.getByTestId('share-button')).toHaveText('Link copied')

  const shareUrl = await page.evaluate(async () => {
    return navigator.clipboard.readText()
  })

  expect(shareUrl).toContain(
    `#share=${encodeShareHashSuffix(SHARED_FILENAME, SHARED_SOURCE)}`,
  )

  await page.goto('/')
  await page.evaluate(() => localStorage.clear())
  // A hash-only change from the current document is a same-document
  // navigation and won't remount the app, unlike a real recipient opening
  // the link fresh — force a full navigation via a blank interstitial page.
  await page.goto('about:blank')
  await page.goto(shareUrl)

  await expect(page.locator('.shared-preview-banner')).toContainText(
    SHARED_FILENAME,
  )

  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  const previewContent = await page.locator('.preview-page').first().innerHTML()
  expect(previewContent).toContain('Shared Score')
})
