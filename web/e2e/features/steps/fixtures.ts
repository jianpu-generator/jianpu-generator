import { test as base, createBdd } from 'playwright-bdd'
import { focusEditor, typeAtEditorEnd } from '../../fileSwitcherHelpers'

export const test = base.extend<{
  focusEditor: () => Promise<void>
  typeAtEditorEnd: (text: string) => Promise<void>
}>({
  focusEditor: async ({ page }, use) => {
    await use(() => focusEditor(page))
  },
  typeAtEditorEnd: async ({ page }, use) => {
    await use((text: string) => typeAtEditorEnd(page, text))
  },
})

export const { Given, When, Then, AfterScenario, BeforeScenario } =
  createBdd(test)
