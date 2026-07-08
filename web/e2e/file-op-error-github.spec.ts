import { expect, test } from '@playwright/test'
import {
  API_PREFIX,
  mockGithubContentsApi,
  OWNER,
} from './github-contents-mock'

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

test('a failed create shows the error modal, resets pending state, and a retry succeeds', async ({
  page,
}) => {
  await mockGithubContentsApi(
    page,
    { 'scores/original.jianpu': SOURCE },
    {
      // Slow enough for the "New" button's pending spinner to be observable.
      mutationDelayMs: 300,
    },
  )

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

  await page.addInitScript(
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
  )

  await page.goto('/')

  const originalTab = page.locator('.file-tab-name', {
    hasText: 'original.jianpu',
  })
  await originalTab.waitFor({ timeout: 15_000 })

  // Positional locator (not `hasText: 'New'`) since its label is swapped for
  // a spinner while the create is pending.
  const newButton = page
    .locator('.file-tab-bar-actions .file-tab-bar-btn')
    .first()
  const activeTabName = page.locator('.file-tab--active .file-tab-name')
  const activeTabBeforeCreate = await activeTabName.textContent()

  await newButton.click()

  // The pending `createFile` call shows a spinner on the "New" button —
  // proves the op actually went in flight before failing.
  await expect(newButton.locator('.file-tab-bar-spinner')).toBeVisible()

  const errorModal = page.getByTestId('error-modal')
  await expect(errorModal).toBeVisible()
  await expect(errorModal).toContainText('Could not create file')
  await expect(page.getByTestId('error-modal-message')).toContainText(
    'Internal Server Error',
  )

  // `finally`'s `setPending(false)` must run on the error path too, not
  // just on success — otherwise the "New" button would be stuck spinning.
  await expect(newButton.locator('.file-tab-bar-spinner')).toHaveCount(0)
  await expect(newButton).toHaveText('New')

  // `setStore` is never called on failure, so no phantom file/tab appears
  // and the active tab is unchanged.
  await expect(
    page.locator('.file-tab-name', { hasText: 'untitled.jianpu' }),
  ).toHaveCount(0)
  await expect(activeTabName).toHaveText(activeTabBeforeCreate ?? '')

  // Close the error modal.
  await page.getByTestId('error-modal').getByRole('button').click()
  await expect(errorModal).toHaveCount(0)

  // The one-shot 500 route has already fired and now falls back to the
  // base mock, so retrying "New" should succeed normally — proving the
  // user can actually recover from the failure.
  await newButton.click()
  await expect(page.locator('.file-tab--active .file-tab-name')).toHaveText(
    'untitled.jianpu',
  )
  await expect(newButton.locator('.file-tab-bar-spinner')).toHaveCount(0)
})
