import { Given } from './fixtures'

Given(
  'the app is loaded and the editor is focused',
  async ({ page, focusEditor }) => {
    await page.goto('/')
    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })
    await focusEditor()
  },
)

Given(
  'GitHub auth is seeded for the mocked owner',
  async ({ seedGithubAuth }) => {
    await seedGithubAuth()
  },
)
