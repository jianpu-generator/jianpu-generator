import { expect, type Page, test } from '@playwright/test'
import { fileSwitcherTrigger } from './fileSwitcherHelpers'

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

async function seedFileStore(
  page: Page,
  filename: string,
  source: string,
  fileId = 'live-test-id',
): Promise<void> {
  await page.addInitScript(
    ({
      key,
      filename,
      source,
      fileId,
    }: {
      key: string
      filename: string
      source: string
      fileId: string
    }) => {
      localStorage.setItem(
        key,
        JSON.stringify({
          active: filename,
          userFiles: { [filename]: source },
          bin: {},
          fileIds: { [filename]: fileId },
        }),
      )
    },
    { key: FILE_STORE_KEY, filename, source, fileId },
  )
}

test('a viewer opening the live link sees the current score immediately, before any owner edit', async ({
  page,
  context,
}) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  await seedFileStore(page, LIVE_FILENAME, LIVE_SOURCE)
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

test('the copied live link carries the filename as a human-readable --suffix, and a viewer opening it still sees the score', async ({
  page,
  context,
}) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  await seedFileStore(page, LIVE_FILENAME, LIVE_SOURCE)
  await page.goto('/')
  await page.getByTestId('go-live-button').click()
  await expect(page.getByTestId('live-link-copied-toast')).toBeVisible()

  const liveUrl = await page.evaluate(async () => {
    return navigator.clipboard.readText()
  })
  expect(liveUrl).toContain(`--${LIVE_FILENAME.replace(/\.jianpu$/, '')}`)

  const viewerPage = await context.newPage()
  await viewerPage.goto(liveUrl)
  await viewerPage.waitForSelector('.preview-page', { timeout: 15_000 })
  const previewContent = await viewerPage
    .locator('.preview-page')
    .first()
    .innerHTML()
  expect(previewContent).toContain('Live Score')
})

test('a viewer opening the live link does not get a ?file= param populated in the URL', async ({
  page,
  context,
}) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  await seedFileStore(page, LIVE_FILENAME, LIVE_SOURCE)
  await page.goto('/')
  await page.getByTestId('go-live-button').click()
  await expect(page.getByTestId('live-link-copied-toast')).toBeVisible()

  const liveUrl = await page.evaluate(async () => {
    return navigator.clipboard.readText()
  })

  const viewerPage = await context.newPage()
  await viewerPage.goto(liveUrl)
  await viewerPage.waitForSelector('.preview-page', { timeout: 15_000 })

  expect(new URL(viewerPage.url()).search).toEqual('')
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
  expect(liveUrl).toMatch(/#live=[0-9A-Za-z_-]{11}(--.+)?$/)

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
  await seedFileStore(page, LIVE_FILENAME, LIVE_SOURCE)
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

test('a viewer importing the live score clears the #live= hash and focuses the imported file', async ({
  page,
  context,
  browser,
}) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  await seedFileStore(page, LIVE_FILENAME, LIVE_SOURCE)
  await page.goto('/')
  await page.getByTestId('go-live-button').click()
  await expect(page.getByTestId('live-link-copied-toast')).toBeVisible()
  const liveUrl = await page.evaluate(() => navigator.clipboard.readText())

  // A separate browser context, since a real viewer is a different browser
  // that doesn't share the owner's localStorage.
  const viewerContext = await browser.newContext()
  const viewerPage = await viewerContext.newPage()
  await viewerPage.goto(liveUrl)
  await viewerPage.waitForSelector('.preview-page', { timeout: 15_000 })

  await viewerPage.getByRole('button', { name: 'Import to my scores' }).click()

  await expect(viewerPage.locator('.shared-preview-banner')).toHaveCount(0)
  expect(new URL(viewerPage.url()).hash).toEqual('')

  await expect(fileSwitcherTrigger(viewerPage)).toContainText(LIVE_FILENAME)
  await viewerContext.close()
})

test('dragging across measures in a live viewer highlights them, even without a mounted editor to round-trip the selection through', async ({
  page,
  context,
}) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  const dragFilename = 'live-drag-test.jianpu'
  const dragSource = [
    '# metadata',
    'title = "Live Drag Score"',
    'max_measures_per_system = 48',
    '',
    '# parts',
    'Melody [M] = notes',
    '',
    '# score',
    '[M] 1_ 1_ 1= 1= 1= 1= 1 -',
    '',
    '[M] 1. 2_ 1_. 2= 0',
    '',
    '[M] 1 - - -',
  ].join('\n')
  await seedFileStore(page, dragFilename, dragSource, 'live-drag-test-id')
  await page.goto('/')
  await page.getByTestId('go-live-button').click()
  await expect(page.getByTestId('live-link-copied-toast')).toBeVisible()
  const liveUrl = await page.evaluate(() => navigator.clipboard.readText())

  const viewerPage = await context.newPage()
  await viewerPage.goto(liveUrl)
  await viewerPage.waitForSelector(
    '[data-tag="measure"][data-measure-index="2"]',
    { timeout: 15_000 },
  )
  // The Parts toolbar only mounts once the worker reports the score's parts,
  // which can land just after the measures above — wait for it so the page
  // layout has settled before measuring bounding boxes below.
  await viewerPage.locator('.part-toggles').first().waitFor({
    state: 'visible',
    timeout: 15_000,
  })
  // `liveViewerActive` (and the `hideEditor` it drives) flips true async,
  // just after the score itself renders — wait for the Editor to actually
  // unmount before dragging, otherwise the drag can race a still-mounted
  // Editor and take the Monaco-selection path this test isn't about.
  await expect(viewerPage.locator('.monaco-editor')).toHaveCount(0)

  const measure0 = viewerPage
    .locator('[data-tag="measure"][data-measure-index="0"]')
    .first()
  const measure2 = viewerPage
    .locator('[data-tag="measure"][data-measure-index="2"]')
    .first()
  await expect(measure0).toBeVisible()
  await expect(measure2).toBeVisible()

  const box0 = await measure0.boundingBox()
  const box2 = await measure2.boundingBox()
  if (!box0 || !box2) {
    throw new Error('Could not get bounding boxes for measures 0 and 2.')
  }

  // Drag from measure 0 to measure 2 in the read-only viewer — a shortcut
  // for selecting every note/rest cell across those measures, same as it is
  // in the editable app (see `Preview.tsx`'s `noteCellsInMeasureRange`).
  await viewerPage.mouse.move(box0.x + box0.width / 2, box0.y + box0.height / 2)
  await viewerPage.mouse.down()
  await viewerPage.mouse.move(
    box2.x + box2.width / 2,
    box2.y + box2.height / 2,
    {
      steps: 10,
    },
  )
  await viewerPage.mouse.up()

  // The selection must still land even on a run where the drag handler has
  // no mounted `editorRef` to push a Monaco selection through (see
  // `useNoteSelection`'s fallback to `notifySelection` directly).
  await expect(
    viewerPage
      .locator('.preview-page [data-testid="measure-highlight"]')
      .first(),
  ).toBeVisible({ timeout: 5_000 })

  // The play-current-measure button lives in AppHeader (not gated on the
  // editor pane), so its label must also pick up the selected range — this
  // reflects `selectedMeasureRange` becoming non-null, independent of
  // whether the (separately-loaded) soundfont asset is ready yet. Unlike the
  // editor-mounted case (where the same drag also lands a Monaco selection
  // and shows "Selection" instead, see `measure-click-selects-notes.spec.ts`),
  // there's no editor here to push a note selection into, so the plain
  // measure-range fallback is what's on screen.
  const playBtn = viewerPage.locator('[data-testid="play-measure-button"]')
  await expect(playBtn).toHaveText(/Measures 1.3/, { timeout: 5_000 })
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
