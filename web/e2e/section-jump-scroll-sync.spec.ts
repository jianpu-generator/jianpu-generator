import { expect, test } from '@playwright/test'

/**
 * Regression test for PENDING_TASK_section-preview-scroll-sync.md: clicking
 * a section button moved the Monaco selection but left the SVG preview
 * scrolled wherever it already was, because the preview's only scroll
 * trigger was gated behind the amber caret-only highlight, and section
 * jumps deliberately select a real (non-empty) range to keep that highlight
 * off. Preview.tsx now also scrolls to the plain `[data-tag="measure"]`
 * group for the selection's first measure when no highlight rect exists.
 *
 * Builds a source with section "A" (measure 0) followed by 59 more
 * measures, then section "B" starting at the last measure — enough content
 * to overflow `.preview-pages` vertically so a scroll is observable.
 */
function buildLongSectionedSource(totalMeasures: number): string {
  const lines = [
    '# metadata',
    'title = "section scroll sync test"',
    '',
    '# parts',
    'Melody [M] = notes',
    '',
    '# score',
    'label="A"',
    '[M] 1 2 3 4',
    '',
  ]
  for (let i = 1; i < totalMeasures - 1; i++) {
    lines.push('[M] 1 2 3 4')
    lines.push('')
  }
  lines.push('label="B"')
  lines.push('[M] 1 2 3 4')
  return lines.join('\n')
}

test('clicking a section button scrolls the SVG preview to that section', async ({
  page,
}) => {
  const totalMeasures = 60
  const source = buildLongSectionedSource(totalMeasures)

  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'section-scroll-sync-test.jianpu',
        userFiles: { 'section-scroll-sync-test.jianpu': src },
        bin: {},
        fileIds: { 'section-scroll-sync-test.jianpu': crypto.randomUUID() },
      }),
    )
  }, source)

  await page.goto('/')
  await page.waitForSelector('button.section-jump-btn', { timeout: 15_000 })

  const lastMeasureIndex = totalMeasures - 1
  await page.waitForSelector(
    `[data-tag="measure"][data-measure-index="${lastMeasureIndex}"]`,
    { timeout: 15_000 },
  )

  const previewPages = page.locator('.preview-pages')
  const scrollTopBefore = await previewPages.evaluate((el) => el.scrollTop)

  await page.locator('button.section-jump-btn', { hasText: 'B' }).click()

  await expect(page.getByTestId('selected-measure-range')).toHaveText(
    `${lastMeasureIndex}-${lastMeasureIndex}`,
    { timeout: 3_000 },
  )

  // The rAF-driven scrollIntoView should move the preview even though a
  // section jump's real (non-empty) selection never populates
  // `highlightedDocuments`/the amber highlight rect.
  await expect
    .poll(async () => previewPages.evaluate((el) => el.scrollTop), {
      timeout: 5_000,
    })
    .not.toBe(scrollTopBefore)

  const lastMeasureGroup = page
    .locator(`[data-tag="measure"][data-measure-index="${lastMeasureIndex}"]`)
    .first()
  await expect(lastMeasureGroup).toBeInViewport({ timeout: 5_000 })
})
