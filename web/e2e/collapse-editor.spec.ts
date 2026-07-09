import { expect, test } from '@playwright/test'

const SOURCE = [
  '# metadata',
  'title = "Collapse Editor Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

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

test('hides the editor pane and expands the preview when toggled', async ({
  page,
}) => {
  const editorPane = page.locator('.pane--editor')
  const toggleButton = page.locator('.pane-divider-toggle')

  await expect(editorPane).toHaveClass(/pane--editor$/)
  const expandedWidth = await editorPane.evaluate(
    (el) => el.getBoundingClientRect().width,
  )
  expect(expandedWidth).toBeGreaterThan(0)

  await toggleButton.click()

  await expect(editorPane).toHaveClass(/pane--editor-collapsed/)
  await expect(toggleButton).toHaveAttribute('title', 'Show editor')

  await expect
    .poll(
      async () => editorPane.evaluate((el) => el.getBoundingClientRect().width),
      { timeout: 3_000 },
    )
    .toBeLessThan(2)
})

test('restores the editor pane when toggled again', async ({ page }) => {
  const editorPane = page.locator('.pane--editor')
  const toggleButton = page.locator('.pane-divider-toggle')

  await toggleButton.click()
  await expect(editorPane).toHaveClass(/pane--editor-collapsed/)

  await toggleButton.click()

  await expect(editorPane).not.toHaveClass(/pane--editor-collapsed/)
  await expect(toggleButton).toHaveAttribute('title', 'Hide editor')

  await expect
    .poll(
      async () => editorPane.evaluate((el) => el.getBoundingClientRect().width),
      { timeout: 3_000 },
    )
    .toBeGreaterThan(50)

  await expect(page.locator('.monaco-editor')).toBeVisible()
})
