import { expect, test } from '@playwright/test'

const FILE_STORE_KEY = 'jianpu:files:v1'
const LIVE_FILENAME = 'live-test.jianpu'
const LIVE_SOURCE = [
  '# metadata',
  'title = "Live Score"',
  '',
  '# parts',
  'Melody = notes',
  '',
  '# score',
  '(time=4/4 key=C4 bpm=120)',
  '1 2 3 4',
].join('\n')

test('a viewer opening the live link sees the current score immediately, before any owner edit', async ({
  page,
  context,
}) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
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
          fileIds: { [filename]: 'live-test-id' },
        }),
      )
    },
    { key: FILE_STORE_KEY, filename: LIVE_FILENAME, source: LIVE_SOURCE },
  )
  await page.goto('/')
  await page.getByTestId('go-live-button').click()
  await expect(page.getByTestId('live-link-copied-toast')).toBeVisible()

  const liveUrl = await page.evaluate(async () => {
    return navigator.clipboard.readText()
  })

  const viewerPage = await context.newPage()
  await viewerPage.goto(liveUrl)

  // No edit was made on the owner's side — the room's initial doc must
  // still arrive as soon as the owner's socket connects.
  await viewerPage.waitForSelector('.preview-page', { timeout: 15_000 })
  const previewContent = await viewerPage
    .locator('.preview-page')
    .first()
    .innerHTML()
  expect(previewContent).toContain('Live Score')
})

test('go live button copies a #live= link and shows a toast, then a dropdown offers copy/stop', async ({
  page,
  context,
}) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  await page.addInitScript(() => {
    localStorage.clear()
  })
  await page.goto('/')
  await page.getByTestId('go-live-button').click()
  await expect(page.getByTestId('live-link-copied-toast')).toBeVisible()

  const liveUrl = await page.evaluate(async () => {
    return navigator.clipboard.readText()
  })
  expect(liveUrl).toMatch(
    /#live=[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
  )

  // Once live, the trigger becomes a dropdown offering Copy / Stop.
  await expect(page.getByTestId('go-live-button')).toHaveText('Live')
  await page.getByTestId('go-live-button').click()
  await expect(page.getByTestId('copy-live-link-button')).toBeVisible()
  await expect(page.getByTestId('stop-live-button')).toBeVisible()

  await page.getByTestId('copy-live-link-button').click()
  await expect(page.getByTestId('live-link-copied-toast')).toBeVisible()
  const copiedAgain = await page.evaluate(() => navigator.clipboard.readText())
  expect(copiedAgain).toEqual(liveUrl)

  await page.getByTestId('go-live-button').click()
  await page.getByTestId('stop-live-button').click()
  await expect(page.getByTestId('stop-live-button')).toHaveCount(0)
  await expect(page.getByTestId('go-live-button')).toHaveText('Go Live')
})

test('stopping live ends the link for viewers, and going live again on the same link revives it', async ({
  page,
  context,
}) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
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
          fileIds: { [filename]: 'live-test-id' },
        }),
      )
    },
    { key: FILE_STORE_KEY, filename: LIVE_FILENAME, source: LIVE_SOURCE },
  )
  await page.goto('/')
  await page.getByTestId('go-live-button').click()
  await expect(page.getByTestId('live-link-copied-toast')).toBeVisible()
  const liveUrl = await page.evaluate(() => navigator.clipboard.readText())

  // A viewer connected *before* the owner stops should lose the score the
  // moment the owner does.
  const viewerPage = await context.newPage()
  await viewerPage.goto(liveUrl)
  await viewerPage.waitForSelector('.preview-page', { timeout: 15_000 })

  await page.getByTestId('go-live-button').click()
  await page.getByTestId('stop-live-button').click()

  await expect(
    viewerPage.getByText('This live session has ended.'),
  ).toBeVisible()
  await expect(viewerPage.locator('.preview-page')).not.toContainText(
    'Live Score',
  )

  // A fresh viewer opening the same link after the stop must not see the
  // score either — the link doesn't quietly stay viewable forever.
  const lateViewerPage = await context.newPage()
  await lateViewerPage.goto(liveUrl)
  await expect(
    lateViewerPage.getByText('This live session has ended.'),
  ).toBeVisible()
  await expect(lateViewerPage.locator('.preview-page')).not.toContainText(
    'Live Score',
  )

  // Going live again reproduces the same link and revives the room.
  await page.getByTestId('go-live-button').click()
  await expect(page.getByTestId('live-link-copied-toast')).toBeVisible()
  const revivedUrl = await page.evaluate(() => navigator.clipboard.readText())
  expect(revivedUrl).toEqual(liveUrl)

  await lateViewerPage.reload()
  await lateViewerPage.waitForSelector('.preview-page', { timeout: 15_000 })
  const previewContent = await lateViewerPage
    .locator('.preview-page')
    .first()
    .innerHTML()
  expect(previewContent).toContain('Live Score')
})

test('re-going-live on the same file reproduces the same link, so it never needs re-sharing', async ({
  page,
  context,
}) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  await page.addInitScript(() => {
    localStorage.clear()
  })
  await page.goto('/')
  await page.getByTestId('go-live-button').click()
  await expect(page.getByTestId('live-link-copied-toast')).toBeVisible()
  const firstUrl = await page.evaluate(() => navigator.clipboard.readText())

  await page.getByTestId('go-live-button').click()
  await page.getByTestId('stop-live-button').click()
  await expect(page.getByTestId('stop-live-button')).toHaveCount(0)

  await page.getByTestId('go-live-button').click()
  await expect(page.getByTestId('live-link-copied-toast')).toBeVisible()
  const secondUrl = await page.evaluate(() => navigator.clipboard.readText())

  expect(secondUrl).toEqual(firstUrl)
})
