import { expect, test } from '@playwright/test'

const SOURCE = [
  '# metadata',
  'title = "Mobile Workspace Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

test.use({ viewport: { width: 375, height: 700 } })

test.beforeEach(async ({ page }) => {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'original.jianpu',
        userFiles: { 'original.jianpu': src },
        bin: {},
        fileIds: { 'original.jianpu': crypto.randomUUID() },
      }),
    )
  }, SOURCE)

  await page.goto('/')
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

test('shows only the preview by default below the mobile breakpoint', async ({
  page,
}) => {
  const editorPane = page.locator('.pane--editor')
  const previewPane = page.locator('.pane--preview')
  const toggleIcon = page.locator('.pane-divider-toggle-icon')

  await expect(editorPane).toHaveClass(/pane--editor-collapsed/)
  await expect(previewPane).not.toHaveClass(/pane--preview-collapsed/)

  const previewHeight = await previewPane.evaluate(
    (el) => el.getBoundingClientRect().height,
  )
  expect(previewHeight).toBeGreaterThan(300)

  // Preview is below the collapsed editor, so the chevron points down.
  await expect(toggleIcon).toHaveCSS('transform', 'matrix(0, -1, 1, 0, 0, 0)')
})

test('toggling swaps to the editor and hides the preview', async ({ page }) => {
  const editorPane = page.locator('.pane--editor')
  const previewPane = page.locator('.pane--preview')
  const toggleButton = page.locator('.pane-divider-toggle')
  const toggleIcon = page.locator('.pane-divider-toggle-icon')

  await toggleButton.click()

  await expect(editorPane).not.toHaveClass(/pane--editor-collapsed/)
  await expect(previewPane).toHaveClass(/pane--preview-collapsed/)
  await expect(page.locator('.monaco-editor')).toBeVisible()

  // Editor is now the visible pane, above the collapsed preview, so the
  // chevron points up.
  await expect(toggleIcon).toHaveCSS('transform', 'matrix(0, 1, -1, 0, 0, 0)')

  await expect
    .poll(
      async () =>
        previewPane.evaluate((el) => el.getBoundingClientRect().height),
      { timeout: 3_000 },
    )
    .toBeLessThan(2)

  await toggleButton.click()

  await expect(editorPane).toHaveClass(/pane--editor-collapsed/)
  await expect(previewPane).not.toHaveClass(/pane--preview-collapsed/)
})
