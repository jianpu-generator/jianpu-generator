import { expect, test } from '@playwright/test'

/**
 * Source with a `# sequence` section that replays "B" twice
 * (`A, B, B`), so the resolved playback order (A, B, B) diverges from
 * written order (A, B) — this is what distinguishes the sequence jump
 * toolbar (SequenceJumpToolbar) from the plain section jump toolbar
 * (SectionJumpToolbar), which only ever shows one button per written label.
 *
 * Lines (1-based):
 *   7:  # sequence
 *   8:  A, B, B
 *   10: # score
 *   11: time=4/4 key=C4 bpm=120 label="A"   ← measure 0
 *   12: 1 2 3 4
 *   14: label="B"                           ← measure 1
 *   15: 5 6 7 1'
 */
const source = [
  '# metadata',
  'title = "test"',
  '',
  '# parts',
  'M = notes',
  '',
  '# sequence',
  'A, B, B',
  '',
  '# score',
  'time=4/4 key=C4 bpm=120 label="A"',
  '1 2 3 4',
  '',
  'label="B"',
  "5 6 7 1'",
].join('\n')

// The section jump toolbar (always present when labels exist) and the
// sequence jump toolbar (only present when a `# sequence` section resolves)
// both render `button.section-jump-btn` elements inside their own
// `[role="toolbar"]`. `SequenceJumpToolbar` is mounted after
// `SectionJumpToolbar` in App.tsx, so it is the second toolbar in the DOM.
function sequenceToolbarButtons(page: import('@playwright/test').Page) {
  return page
    .locator('[role="toolbar"]')
    .nth(1)
    .locator('button.section-jump-btn')
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'sequence-test.jianpu',
        userFiles: { 'sequence-test.jianpu': src },
        bin: {},
        fileIds: { 'sequence-test.jianpu': crypto.randomUUID() },
      }),
    )
  }, source)

  await page.goto('/')

  // The play-from-current-measure button only renders once audio is
  // available (i.e. the worker has finished processing the source).
  await page.waitForSelector(
    '[data-testid="play-from-current-measure-button"]',
    { timeout: 15_000 },
  )
  // Sequence buttons appear once the worker returns resolved sequence
  // entries, which lands slightly after the play button.
  await expect(sequenceToolbarButtons(page)).toHaveCount(3, {
    timeout: 15_000,
  })
})

test('sequence jump toolbar renders one button per resolved sequence entry, in playback order', async ({
  page,
}) => {
  const buttons = sequenceToolbarButtons(page)
  await expect(buttons.nth(0)).toHaveText('A')
  await expect(buttons.nth(1)).toHaveText('B')
  await expect(buttons.nth(2)).toHaveText('B')
})

test('clicking the "A" entry enables playback from measure 1', async ({
  page,
}) => {
  const playBtn = page.getByTestId('play-from-current-measure-button')
  await expect(playBtn).toBeDisabled()

  await sequenceToolbarButtons(page).nth(0).click()

  // Selecting a sequence entry updates the aria-label immediately; whether
  // the button is actually clickable also depends on the (real, ~30 MB)
  // soundfont finishing its load, which is covered separately in
  // sequence-jump-toolbar-play.spec.ts.
  await expect(playBtn).toHaveAttribute(
    'aria-label',
    'Play sequence from Measure 1',
  )
})

test('clicking the first "B" entry enables playback from measure 2', async ({
  page,
}) => {
  const playBtn = page.getByTestId('play-from-current-measure-button')

  await sequenceToolbarButtons(page).nth(1).click()

  await expect(playBtn).toHaveAttribute(
    'aria-label',
    'Play sequence from Measure 2',
  )
})

// The two "B" buttons (indices 1 and 2) both map to the same written measure
// (index 1), since `# sequence` just replays that span twice. Clicking one
// must highlight only that specific occurrence — proving selection is keyed
// by sequence position, not by label or by measure range.
test('clicking the second "B" occurrence highlights only that button, not the first "B"', async ({
  page,
}) => {
  const buttons = sequenceToolbarButtons(page)
  const firstB = buttons.nth(1)
  const secondB = buttons.nth(2)

  await secondB.click()

  await expect(secondB).toHaveClass(/section-jump-btn--dragging/)
  await expect(firstB).not.toHaveClass(/section-jump-btn--dragging/)
})

// Dragging from one entry button to another selects the merged range spanning
// both, via useSequenceNavigation's handleSequenceEntryRangeSelect — mirrors
// the drag coverage in section-jump-select.spec.ts for the (separate) section
// jump toolbar.
test('dragging from the "A" entry to the "B" entry selects the merged range and highlights both buttons', async ({
  page,
}) => {
  const buttons = sequenceToolbarButtons(page)
  const buttonA = buttons.nth(0)
  const buttonB = buttons.nth(1)

  const fromBox = await buttonA.boundingBox()
  const toBox = await buttonB.boundingBox()
  if (!fromBox || !toBox) {
    throw new Error('Could not get bounding boxes for sequence entry buttons.')
  }

  await page.mouse.move(
    fromBox.x + fromBox.width / 2,
    fromBox.y + fromBox.height / 2,
  )
  await page.mouse.down()
  await expect(buttonA).toHaveClass(/section-jump-btn--dragging/, {
    timeout: 3_000,
  })
  await page.mouse.move(toBox.x + toBox.width / 2, toBox.y + toBox.height / 2, {
    steps: 10,
  })
  await expect(buttonB).toHaveClass(/section-jump-btn--dragging/, {
    timeout: 3_000,
  })
  await page.mouse.up()

  // The merged range still starts at measure 0 (the "A" entry's start), so
  // playback is enabled from measure 1, and both buttons stay highlighted as
  // the active (not merely dragging) selection.
  const playBtn = page.getByTestId('play-from-current-measure-button')
  await expect(playBtn).toHaveAttribute(
    'aria-label',
    'Play sequence from Measure 1',
  )
  await expect(buttonA).toHaveClass(/section-jump-btn--dragging/)
  await expect(buttonB).toHaveClass(/section-jump-btn--dragging/)
})
