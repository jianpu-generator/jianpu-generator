import { test as base, createBdd } from 'playwright-bdd'
import { focusEditor, typeAtEditorEnd } from '../../fileSwitcherHelpers'
import { OWNER } from '../../github-contents-mock'

export const test = base.extend<{
  focusEditor: () => Promise<void>
  typeAtEditorEnd: (text: string) => Promise<void>
  seedGithubAuth: () => Promise<void>
}>({
  focusEditor: async ({ page }, use) => {
    await use(() => focusEditor(page))
  },
  typeAtEditorEnd: async ({ page }, use) => {
    await use((text: string) => typeAtEditorEnd(page, text))
  },
  // Must run (via addInitScript) before `page.goto`, so scenarios call this
  // fixture ahead of the "app is loaded" step rather than it being implied.
  seedGithubAuth: async ({ page }, use) => {
    await use(() =>
      page.addInitScript(
        ({ owner }) => {
          localStorage.setItem(
            'jianpu:storage-backend:v1',
            JSON.stringify({ backend: 'github', github: { owner } }),
          )
          localStorage.setItem(
            'jianpu:github-auth:v1',
            JSON.stringify({ token: 'fake-token', scopes: ['repo'] }),
          )
        },
        { owner: OWNER },
      ),
    )
  },
})

export const { Given, When, Then, AfterScenario } = createBdd(test)
