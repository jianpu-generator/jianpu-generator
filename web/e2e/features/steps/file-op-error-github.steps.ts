import { expect } from '@playwright/test'
import {
  fileSwitcherTrigger,
  openFileActions,
  openFileList,
} from '../../fileSwitcherHelpers'
import { API_PREFIX, mockGithubContentsApi } from '../../github-contents-mock'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  '# metadata',
  'title = "File Op Error Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

let activeTabBeforeCreate: string | null = null
const newButton = ({ page }: { page: import('@playwright/test').Page }) =>
  page.locator('.export-menu-item').first()

Given(
  'the GitHub repo is seeded with a file named {string} for a failing create',
  async ({ page }, path: string) => {
    await mockGithubContentsApi(
      page,
      { [path]: SOURCE },
      {
        // Slow enough for the "New" button's pending spinner to be observable.
        mutationDelayMs: 300,
      },
    )
  },
)

Given('the first create PUT will fail with a 500 error', async ({ page }) => {
  // Registered after the base mock above, so Playwright routes it first:
  // it fails the first PUT with a 500, then `route.fallback()`s to the base
  // mock for every request after (including a retried PUT), so the mock
  // behaves normally once this one-shot failure has fired.
  let putCount = 0
  await page.route(`${API_PREFIX}**`, async (route) => {
    const request = route.request()
    if (request.method() === 'PUT') {
      putCount += 1
      if (putCount === 1) {
        // Matches the base mock's `mutationDelayMs` above so the "New"
        // button's spinner has time to render before this fails, same as
        // it would for a real slow-then-failing request.
        await new Promise((resolve) => setTimeout(resolve, 300))
        return route.fulfill({
          status: 500,
          json: { message: 'Internal Server Error' },
        })
      }
    }
    return route.fallback()
  })
})

When(
  'the app loads the GitHub-backed file list for a failing create',
  async ({ page }) => {
    await page.goto('/')

    await openFileList(page)
    const originalTab = page.locator('.file-tab-name', {
      hasText: 'original',
    })
    await originalTab.waitFor({ timeout: 15_000 })
  },
)

Given('I remember the currently active tab name', async ({ page }) => {
  activeTabBeforeCreate = await fileSwitcherTrigger(page).textContent()
})

When(
  'I click the {string} button to create a file that will fail',
  async ({ page }, label: string) => {
    expect(label).toBe('New')
    // Positional locator (not `hasText: 'New'`) since its label is swapped for
    // a spinner while the create is pending.
    await openFileActions(page)
    await newButton({ page }).click()
  },
)

Then(
  'the new-file button shows a pending spinner before the create fails',
  async ({ page }) => {
    // Clicking "New" closes the "⋯" dropdown immediately; reopen it to
    // observe the pending `createFile` call's spinner on the "New" button —
    // proves the op actually went in flight before failing.
    await openFileActions(page)
    await expect(
      newButton({ page }).locator('.file-tab-bar-spinner'),
    ).toBeVisible()
  },
)

Then(
  'the error modal is shown with message {string} containing {string}',
  async ({ page }, title: string, detail: string) => {
    const errorModal = page.getByTestId('error-modal')
    await expect(errorModal).toBeVisible()
    await expect(errorModal).toContainText(title)
    await expect(page.getByTestId('error-modal-message')).toContainText(detail)
  },
)

When('I close the error modal', async ({ page }) => {
  // Close the error modal before interacting with anything underneath it —
  // it's a real overlay that blocks pointer events on the rest of the page.
  await page.getByTestId('error-modal').getByRole('button').click()
  await expect(page.getByTestId('error-modal')).toHaveCount(0)
})

Then(
  'the new-file button spinner clears and its label resets to {string}',
  async ({ page }, label: string) => {
    // `finally`'s `setPending(false)` must run on the error path too, not
    // just on success — otherwise the "New" button would be stuck spinning.
    // The dropdown closes once `onCreate` settles (success or failure), so
    // reopen it to observe the reset state.
    await openFileActions(page)
    await expect(
      newButton({ page }).locator('.file-tab-bar-spinner'),
    ).toHaveCount(0)
    await expect(newButton({ page })).toHaveText(label)
  },
)

Then('no {string} tab exists', async ({ page }, name: string) => {
  // `setStore` is never called on failure, so no phantom file/tab appears
  // and the active tab is unchanged.
  await openFileList(page)
  await expect(page.locator('.file-tab-name', { hasText: name })).toHaveCount(0)
})

Then(
  'the active tab is unchanged from before the failed create',
  async ({ page }) => {
    await expect(fileSwitcherTrigger(page)).toHaveText(
      activeTabBeforeCreate ?? '',
    )
  },
)

When(
  'I retry the {string} button in the file actions menu',
  async ({ page }, label: string) => {
    expect(label).toBe('New')
    // The one-shot 500 route has already fired and now falls back to the
    // base mock, so retrying "New" should succeed normally — proving the
    // user can actually recover from the failure.
    await openFileActions(page)
    await newButton({ page }).click()
  },
)

Then(
  'the retried create succeeds and the active tab becomes {string}',
  async ({ page }, name: string) => {
    await expect(fileSwitcherTrigger(page)).toContainText(name)
  },
)

Then('the new-file button has no pending spinner', async ({ page }) => {
  await openFileActions(page)
  await expect(
    newButton({ page }).locator('.file-tab-bar-spinner'),
  ).toHaveCount(0)
})
