import { expect, test } from '@playwright/test'

/**
 * Regression test: dragging to a `# sequence` entry that resolves far away
 * from the drag's other end (here, a repeated label whose written measures
 * sit much earlier in the source than the entry it's selected alongside)
 * must scroll the editor to the entry the user actually dragged *to*, not
 * always whichever entry has the smaller sequence index. Both ends are
 * still genuinely selected (see sequence-jump-select-out-of-order.spec.ts
 * for that assertion) — this covers which one ends up visible when they're
 * too far apart in the source to both fit on screen at once.
 *
 * Padded with filler measures so "Intro" (lines 11-12) and "C" (near the
 * bottom) can't both land in the same viewport.
 */
const fillerMeasures = Array.from(
  { length: 60 },
  (_, i) => `label="Filler${i}"\n1 2 3 4`,
).join('\n\n')

const source = [
  '# metadata',
  'title = "test"',
  '',
  '# parts',
  'M = notes',
  '',
  '# sequence',
  'Intro, C, Intro',
  '',
  '# score',
  'time=4/4 key=C4 bpm=120 label="Intro"',
  '1 2 3 4',
  '',
  fillerMeasures,
  '',
  'label="C"',
  "1' 7 6 5",
].join('\n')

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
        active: 'sequence-reveal-target-test.jianpu',
        userFiles: { 'sequence-reveal-target-test.jianpu': src },
        bin: {},
        fileIds: {
          'sequence-reveal-target-test.jianpu': crypto.randomUUID(),
        },
      }),
    )
  }, source)

  await page.goto('/')
  await expect(sequenceToolbarButtons(page)).toHaveCount(3, {
    timeout: 15_000,
  })
})

test('dragging from "C" to the repeated "Intro" reveals Intro\'s written lines, not C\'s', async ({
  page,
}) => {
  const buttons = sequenceToolbarButtons(page)

  // Chain-order indices: 0 = Intro (first), 1 = C, 2 = Intro (repeat,
  // resolving to the same written lines as index 0). Drag from "C" (1) to
  // the repeated "Intro" (2) — the drag ends on the far-away entry.
  await buttons.nth(1).hover()
  await page.mouse.down()
  await buttons.nth(2).hover()
  await page.mouse.up()

  await expect
    .poll(
      () =>
        page.evaluate(() => {
          const monacoApi = (
            window as unknown as { monaco: typeof import('monaco-editor') }
          ).monaco
          const ed = monacoApi.editor.getEditors()[0]
          return ed
            ?.getVisibleRanges()
            .some((r) => r.startLineNumber <= 12 && r.endLineNumber >= 11)
        }),
      { timeout: 3_000 },
    )
    .toBe(true)
})
