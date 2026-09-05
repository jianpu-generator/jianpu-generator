import { expect } from '@playwright/test'
import { clickAndClickSelect, stableBoundingBox } from '../../dragSelectHelpers'
import { fileSwitcherTrigger } from '../../fileSwitcherHelpers'
import { Then, When } from './fixtures'
import {
  LIVE_FILENAME,
  liveShareButtonState as state,
} from './live-share-button-state'

When(
  'a separate browser context opens the copied live link as a viewer',
  async ({ browser }) => {
    if (!state.liveUrl) throw new Error('liveUrl was not captured yet')
    // A separate browser context, since a real viewer is a different browser
    // that doesn't share the owner's localStorage.
    state.viewerContext = await browser.newContext()
    state.viewerPage = await state.viewerContext.newPage()
    await state.viewerPage.goto(state.liveUrl)
    await state.viewerPage.waitForSelector('.preview-page', {
      timeout: 15_000,
    })
  },
)

When('the viewer clicks {string}', async ({}, label: string) => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  await state.viewerPage.getByRole('button', { name: label }).click()
})

Then("the viewer's shared preview banner is gone", async () => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  await expect(state.viewerPage.locator('.shared-preview-banner')).toHaveCount(
    0,
  )
})

Then("the viewer's page URL has no hash", async () => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  expect(new URL(state.viewerPage.url()).hash).toEqual('')
})

Then("the viewer's file switcher shows the live filename", async () => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  await expect(fileSwitcherTrigger(state.viewerPage)).toContainText(
    LIVE_FILENAME.replace(/\.jianpu$/, ''),
  )
})

When(
  'a viewer opens the copied live link in a new page and waits for measures to render',
  async ({ context }) => {
    if (!state.liveUrl) throw new Error('liveUrl was not captured yet')
    state.viewerPage = await context.newPage()
    await state.viewerPage.goto(state.liveUrl)
    await state.viewerPage.waitForSelector(
      '[data-tag="measure"][data-measure-index="2"]',
      { timeout: 15_000 },
    )
  },
)

Then(
  "the viewer's parts toolbar is visible and no Monaco editor is mounted",
  async () => {
    if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
    // The Parts toolbar only mounts once the worker reports the score's parts,
    // which can land just after the measures above — wait for it so the page
    // layout has settled before measuring bounding boxes below.
    await state.viewerPage.locator('.part-toggles').first().waitFor({
      state: 'visible',
      timeout: 15_000,
    })
    // `liveViewerActive` (and the `hideEditor` it drives) flips true async,
    // just after the score itself renders — wait for the Editor to actually
    // unmount before dragging, otherwise the drag can race a still-mounted
    // Editor and take the Monaco-selection path this test isn't about.
    await expect(state.viewerPage.locator('.monaco-editor')).toHaveCount(0)
  },
)

When(
  'the viewer drags from measure {int} to measure {int}',
  async ({}, fromIndex: number, toIndex: number) => {
    if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
    const measureFrom = state.viewerPage
      .locator(`[data-tag="measure"][data-measure-index="${fromIndex}"]`)
      .first()
    const measureTo = state.viewerPage
      .locator(`[data-tag="measure"][data-measure-index="${toIndex}"]`)
      .first()
    await expect(measureFrom).toBeVisible()
    await expect(measureTo).toBeVisible()

    const boxFrom = await stableBoundingBox(measureFrom)
    const boxTo = await stableBoundingBox(measureTo)
    if (!boxFrom || !boxTo) {
      throw new Error(
        `Could not get bounding boxes for measures ${fromIndex} and ${toIndex}.`,
      )
    }

    // Click-and-click from measure 0 to measure 2 in the read-only viewer —
    // a shortcut for selecting every note/rest cell across those measures,
    // same as it is in the editable app (see `previewSelection.ts`'s
    // `noteCellsInMeasureRange`).
    await clickAndClickSelect(
      state.viewerPage,
      boxFrom.x + boxFrom.width / 2,
      boxFrom.y + boxFrom.height / 2,
      boxTo.x + boxTo.width / 2,
      boxTo.y + boxTo.height / 2,
    )
  },
)

When('the viewer taps the first note', async ({}) => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  const noteRect = state.viewerPage
    .locator('rect[data-variant="note-click-target-rect"]')
    .first()
  await expect(noteRect).toBeVisible()
  const box = await stableBoundingBox(noteRect)
  if (!box) throw new Error('Could not get bounding box for the first note.')
  // A single click-and-click at the same point selects just that one note
  // (see `clickAndClickSelect`'s doc comment) — the regression this guards
  // against is a plain tap also painting the whole-measure amber overlay in
  // this no-mounted-editor viewer (see `fireCommit`'s and
  // `useMeasureRangeSelection`'s doc comments).
  await clickAndClickSelect(
    state.viewerPage,
    box.x + box.width / 2,
    box.y + box.height / 2,
    box.x + box.width / 2,
    box.y + box.height / 2,
  )
})

When('the viewer taps a bar line', async ({}) => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  const barLineRect = state.viewerPage
    .locator('rect[data-variant="bar-line-click-target-rect"]')
    .first()
  await expect(barLineRect).toBeVisible()
  const box = await stableBoundingBox(barLineRect)
  if (!box) throw new Error('Could not get bounding box for the bar line.')
  // A bar-line tap always anchors 'measure' mode (see
  // `previewClickHandler.ts`'s `handleAnchorClick`) — the mobile bug this
  // guards against: tapping a bar line in this no-mounted-editor viewer
  // still painted the amber whole-measure overlay after 45a815d fixed the
  // same overlay for plain note/lyric taps (see
  // `useMeasureRangeSelection`'s doc comment).
  await clickAndClickSelect(
    state.viewerPage,
    box.x + box.width / 2,
    box.y + box.height / 2,
    box.x + box.width / 2,
    box.y + box.height / 2,
  )
})

Then("the viewer's tapped note is highlighted", async () => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  await expect(
    state.viewerPage.locator('[data-tag="note"][data-note-drag-selected]'),
  ).toHaveCount(1, { timeout: 5_000 })
})

Then("the viewer's measure highlight is not shown", async () => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  // The amber whole-measure background is reserved for an actual Monaco
  // caret (see `useMeasureRangeSelection`'s no-mounted-editor branch) — this
  // no-editor viewer never mounts one, so no SVG gesture here (a single-note
  // tap or a multi-measure drag) ever paints it, even though the drag still
  // updates the play-measure button's range (see the sibling step above).
  await expect(
    state.viewerPage.locator('.preview-page [data-testid="measure-highlight"]'),
  ).toHaveCount(0)
})

Then(
  "the viewer's play-measure button reads {string}",
  async ({}, label: string) => {
    if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
    // The play-current-measure button lives in AppHeader (not gated on the
    // editor pane), so its label must also pick up the selected range — this
    // reflects `selectedMeasureRange` becoming non-null, independent of
    // whether the (separately-loaded) soundfont asset is ready yet. Unlike the
    // editor-mounted case (where the same drag also lands a Monaco selection
    // and shows "Selection" instead, see `measure-click-selects-notes.spec.ts`),
    // there's no editor here to push a note selection into, so the plain
    // measure-range fallback is what's on screen.
    const playBtn = state.viewerPage.locator(
      '[data-testid="play-measure-button"]',
    )
    const pattern = new RegExp(label.replace('-', '.'))
    await expect(playBtn).toHaveText(pattern, { timeout: 5_000 })
  },
)
