import { expect } from '@playwright/test'
import { Then, When } from './fixtures'
import { openSyncedTab } from './synced-share-button.steps'
import { syncedShareButtonState as state } from './synced-share-button-state'

When(
  'a late viewer opens the copied sync link in a new page',
  async ({ browser }) => {
    if (!state.syncedShareLink)
      throw new Error('syncedShareLink was not captured yet')
    // A fresh viewer opening the same link after the stop must not see the
    // score either — the link doesn't quietly stay viewable forever. An
    // isolated browser context, not `context.newPage()` -- see
    // `viewerContext`'s comment in `synced-share-button-state.ts`.
    state.lateViewerContext = await browser.newContext()
    state.lateViewerPage = await state.lateViewerContext.newPage()
    await state.lateViewerPage.goto(state.syncedShareLink)
  },
)

Then('the late viewer sees {string}', async ({}, text: string) => {
  if (!state.lateViewerPage)
    throw new Error('lateViewerPage was not opened yet')
  await expect(state.lateViewerPage.getByText(text)).toBeVisible()
})

Then(
  "the late viewer's preview no longer contains {string}",
  async ({}, text: string) => {
    if (!state.lateViewerPage)
      throw new Error('lateViewerPage was not opened yet')
    await expect(
      state.lateViewerPage.locator('.preview-page'),
    ).not.toContainText(text)
  },
)

When('the owner clicks {string} again', async ({ page }, label: string) => {
  expect(label).toBe('Sync')
  // Syncing again reproduces the same link and revives the share. Usually
  // the modal is already open on the Synced-link tab (only the viewer's
  // page reloaded in between, not the owner's), now back in its "not
  // synced" state after the stop, so Start Sync is clickable directly --
  // except for a scenario that closed the modal in between to interact with
  // the header (e.g. switching to a different file tab), which reopening
  // here accounts for.
  const startSync = page.getByTestId('share-modal-start-sync')
  if ((await startSync.count()) === 0) await openSyncedTab(page)
  await page.getByTestId('share-modal-start-sync').click()
})

Then(
  'the revived sync link is identical to the original link',
  async ({ page }) => {
    if (!state.originalSyncedLink)
      throw new Error('originalSyncedLink was not captured yet')
    const revivedUrl = await page.evaluate(() => navigator.clipboard.readText())
    expect(revivedUrl).toEqual(state.originalSyncedLink)
  },
)

When('the late viewer reloads the page', async () => {
  if (!state.lateViewerPage)
    throw new Error('lateViewerPage was not opened yet')
  await state.lateViewerPage.reload()
  await state.lateViewerPage.waitForSelector('.preview-page', {
    timeout: 15_000,
  })
})

Then(
  "the late viewer's preview contains {string}",
  async ({}, text: string) => {
    if (!state.lateViewerPage)
      throw new Error('lateViewerPage was not opened yet')
    await expect(
      state.lateViewerPage.locator('.preview-page').first(),
    ).toContainText(text)
  },
)
