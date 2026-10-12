import { expect } from '@playwright/test'
import { Then, When } from './fixtures'

// The merge editor and sync panel steps live in
// `cloud-outbox-conflicts.steps.ts` / `cloud-outbox-panel.steps.ts`.

When(
  'I resolve the conflict markers in the merge editor keeping my version',
  async ({ page }) => {
    await page.evaluate(() => {
      const model = window.monaco?.editor
        .getDiffEditors()[0]
        ?.getModifiedEditor()
        .getModel()
      if (!model) throw new Error('no merge editor model')
      // Keep the first (mine) side of every conflict block.
      model.setValue(
        model
          .getValue()
          .replace(
            /<<<<<<<[^\n]*\n([\s\S]*?)(?:\|\|\|\|\|\|\|[^\n]*\n[\s\S]*?)?=======\n[\s\S]*?>>>>>>>[^\n]*\n?/g,
            '$1',
          ),
      )
    })
  },
)

Then(
  'the conflict status badge shows exactly {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('save-status-badge')).toHaveText(text, {
      timeout: 15_000,
    })
  },
)

Then('the editor still contains {string}', async ({ page }, text: string) => {
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(text)
})

Then(
  'the editor now shows the remote content {string}',
  async ({ page }, text: string) => {
    await expect(page.locator('.monaco-editor .view-lines')).toContainText(
      text,
      { timeout: 10_000 },
    )
  },
)

Then(
  'the editor no longer contains {string}',
  async ({ page }, text: string) => {
    await expect(page.locator('.monaco-editor .view-lines')).not.toContainText(
      text,
    )
  },
)
