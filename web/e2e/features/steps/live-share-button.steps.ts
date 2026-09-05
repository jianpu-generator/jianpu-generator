import { expect } from '@playwright/test'
import { AfterScenario, Given, Then, When } from './fixtures'
import {
  LIVE_FILENAME,
  LIVE_SOURCE,
  seedFileStore,
  liveShareButtonState as state,
} from './live-share-button-state'

// `viewerContext` comes from `browser.newContext()`, which — unlike the
// per-test `context`/`page` fixtures — Playwright never closes on its own,
// so every scenario that opens one must close it itself once done. Doing
// that here (rather than in whichever assertion step happens to run last)
// means it's not tied to a particular scenario's step order.
AfterScenario(async () => {
  if (state.viewerContext) {
    await state.viewerContext.close()
    state.viewerContext = undefined
  }
})

Given('clipboard permissions are granted', async ({ context }) => {
  state.liveUrl = undefined
  state.originalLiveUrl = undefined
  state.viewerPage = undefined
  state.lateViewerPage = undefined
  state.viewerContext = undefined
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
})

Given('the file store is seeded with the live score', async ({ page }) => {
  await seedFileStore(page, LIVE_FILENAME, LIVE_SOURCE)
})

Given(
  'the file store is seeded with a multi-measure live drag score',
  async ({ page }) => {
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
  },
)

Given(
  'the file store is seeded with a two-section live score',
  async ({ page }) => {
    // Mirrors `section-jump-select.steps.ts`'s two-section fixture, but with
    // `[M]`-prefixed lines (not the bare `M = notes` shorthand) so each note
    // gets its own click-target rect — needed here so a bar-line tap
    // actually paints a per-note highlight to prove stale afterward (see
    // this hook's own doc comment: the bare shorthand renders no individual
    // note click targets at all, so `applyPersistedNoteHighlights` would
    // have nothing to flag either way, silently passing regardless of the
    // bug this scenario guards against).
    const sectionFilename = 'live-section-test.jianpu'
    const sectionSource = [
      '# metadata',
      'title = "Live Section Score"',
      '',
      '# parts',
      'Melody [M] = notes',
      '',
      '# score',
      'time=4/4 key=C4 bpm=120 label="A"',
      '[M] 1 2 3 4',
      '',
      "[M] 5 6 7 1'",
      '',
      'label="B"',
      "[M] 1' 7 6 5",
      '',
      '[M] 4 3 2 1',
    ].join('\n')
    await seedFileStore(
      page,
      sectionFilename,
      sectionSource,
      'live-section-test-id',
    )
  },
)

Given('local storage is cleared', async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.clear()
  })
})

When(
  'the owner loads the app and clicks {string}',
  async ({ page }, label: string) => {
    expect(label).toBe('Go Live')
    await page.goto('/')
    await page.getByTestId('go-live-button').click()
  },
)

Then('a live-link-copied toast is shown', async ({ page }) => {
  await expect(page.getByTestId('live-link-copied-toast')).toBeVisible()
  state.liveUrl = await page.evaluate(async () => {
    return navigator.clipboard.readText()
  })
  if (state.originalLiveUrl === undefined) {
    state.originalLiveUrl = state.liveUrl
  }
})

When(
  'a viewer opens the copied live link in a new page',
  async ({ context }) => {
    if (!state.liveUrl) throw new Error('liveUrl was not captured yet')
    state.viewerPage = await context.newPage()
    await state.viewerPage.goto(state.liveUrl)

    // No edit was made on the owner's side — the room's initial doc must
    // still arrive as soon as the owner's socket connects.
    await state.viewerPage.waitForSelector('.preview-page', {
      timeout: 15_000,
    })
  },
)

Then("the viewer's preview contains {string}", async ({}, text: string) => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  const previewContent = await state.viewerPage
    .locator('.preview-page')
    .first()
    .innerHTML()
  expect(previewContent).toContain(text)
})

Then(
  'the copied live link contains the filename as a human-readable suffix',
  async () => {
    if (!state.liveUrl) throw new Error('liveUrl was not captured yet')
    expect(state.liveUrl).toContain(
      `--${LIVE_FILENAME.replace(/\.jianpu$/, '')}`,
    )
  },
)

Then("the viewer's page URL has no query string", async () => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  expect(new URL(state.viewerPage.url()).search).toEqual('')
})

Then('the copied live link matches the live URL hash format', async () => {
  if (!state.liveUrl) throw new Error('liveUrl was not captured yet')
  expect(state.liveUrl).toMatch(/#live=[0-9A-Za-z_-]{11}(--.+)?$/)
})

Then(
  'the go-live button now reads {string}',
  async ({ page }, text: string) => {
    // Once live, the trigger becomes a dropdown offering Copy / Stop.
    await expect(page.getByTestId('go-live-button')).toHaveText(text)
  },
)

When('the owner clicks the go-live button again', async ({ page }) => {
  await page.getByTestId('go-live-button').click()
})

Then(
  'the copy-live-link and stop-live buttons are visible',
  async ({ page }) => {
    await expect(page.getByTestId('copy-live-link-button')).toBeVisible()
    await expect(page.getByTestId('stop-live-button')).toBeVisible()
  },
)

When('the owner clicks the copy-live-link button', async ({ page }) => {
  await page.getByTestId('copy-live-link-button').click()
})

Then('the copied link is unchanged from before', async ({ page }) => {
  if (!state.liveUrl) throw new Error('liveUrl was not captured yet')
  const copiedAgain = await page.evaluate(() => navigator.clipboard.readText())
  expect(copiedAgain).toEqual(state.liveUrl)
})

When(
  'the owner clicks the go-live button and then the stop-live button',
  async ({ page }) => {
    await page.getByTestId('go-live-button').click()
    await page.getByTestId('stop-live-button').click()
  },
)

Then('the stop-live button disappears', async ({ page }) => {
  await expect(page.getByTestId('stop-live-button')).toHaveCount(0)
})

Then('the go-live button reads {string}', async ({ page }, text: string) => {
  await expect(page.getByTestId('go-live-button')).toHaveText(text)
})

Then('the viewer sees the preview page', async () => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  await state.viewerPage.waitForSelector('.preview-page', { timeout: 15_000 })
})

Then('the viewer sees {string}', async ({}, text: string) => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  // A viewer connected *before* the owner stops should lose the score the
  // moment the owner does.
  await expect(state.viewerPage.getByText(text)).toBeVisible()
})

Then(
  "the viewer's preview no longer contains {string}",
  async ({}, text: string) => {
    if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
    await expect(state.viewerPage.locator('.preview-page')).not.toContainText(
      text,
    )
  },
)

When(
  'a late viewer opens the copied live link in a new page',
  async ({ context }) => {
    if (!state.liveUrl) throw new Error('liveUrl was not captured yet')
    // A fresh viewer opening the same link after the stop must not see the
    // score either — the link doesn't quietly stay viewable forever.
    state.lateViewerPage = await context.newPage()
    await state.lateViewerPage.goto(state.liveUrl)
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
  expect(label).toBe('Go Live')
  // Going live again reproduces the same link and revives the room.
  await page.getByTestId('go-live-button').click()
})

Then(
  'the revived live link is identical to the original link',
  async ({ page }) => {
    if (!state.originalLiveUrl)
      throw new Error('originalLiveUrl was not captured yet')
    const revivedUrl = await page.evaluate(() => navigator.clipboard.readText())
    expect(revivedUrl).toEqual(state.originalLiveUrl)
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
    const previewContent = await state.lateViewerPage
      .locator('.preview-page')
      .first()
      .innerHTML()
    expect(previewContent).toContain(text)
  },
)
