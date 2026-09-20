import type { Browser, Page } from '@playwright/test'
import { expect } from '@playwright/test'
import { fileSwitcherTrigger, openFileList } from '../../fileSwitcherHelpers'
import { mockGithubContentsApi, OWNER } from '../../github-contents-mock'
import { syncedShareIdentityTokenFor } from '../../mockGithubIdentity.mjs'
import { AfterScenario, BeforeScenario, Given, Then, When } from './fixtures'
import { openSyncedTab } from './synced-share-button.steps'
import { syncedShareButtonState as sharedState } from './synced-share-button-state'
import {
  idempotentSharingSource,
  resetIdempotentCreateState,
  idempotentCreateState as state,
} from './synced-share-idempotent-create-state'

// Node-process-level state shared across the `Given` steps below --
// accumulates every file seeded so far *within one scenario* (scenario 3
// seeds two), since each `mockGithubContentsApi` call registers a fresh
// `page.route()` handler that fully shadows any earlier one for the same
// URL pattern (no `route.fallback()` call) -- passing the whole accumulated
// map on every call keeps the latest (and only effective) registration
// serving every file seeded so far. Reset via `BeforeScenario`, not at the
// top of the seed step itself, since resetting there would also wipe out
// scenario 3's *first* seed call when its second one runs.
let seededFiles: Record<string, string> = {}

BeforeScenario(async () => {
  seededFiles = {}
  resetIdempotentCreateState()
})

AfterScenario(async () => {
  if (state.secondContext) {
    await state.secondContext.close()
    state.secondContext = undefined
  }
})

Given(
  'the GitHub repo is seeded with a file named {string} for idempotent sharing',
  async ({ page }, path: string) => {
    seededFiles[path] = idempotentSharingSource(path)
    await mockGithubContentsApi(page, seededFiles)
  },
)

When(
  'the app loads the GitHub-backed file list for idempotent sharing',
  async ({ page }) => {
    await page.goto('/')
    await openFileList(page)
  },
)

/** Closes the share modal if a prior "Sync" click in this scenario left it
 * open -- Radix's Dialog overlay intercepts pointer events on the rest of
 * the page while open, so the header's file-switcher trigger (needed by
 * both this step and the rename step below) can't be clicked until it's
 * gone. Closing it only hides the UI; it doesn't stop an active sync (that
 * lives in React state / localStorage, untouched by the modal's own
 * open/closed state), so a later step re-opens it via `openSyncedTab`
 * whenever it needs to check or use it again. */
async function closeShareModalIfOpen(page: Page): Promise<void> {
  const modal = page.getByTestId('share-modal')
  if ((await modal.count()) === 0) return
  await page.keyboard.press('Escape')
  await modal.waitFor({ state: 'hidden' })
}

When(
  'I select the {string} tab to test idempotent sharing',
  async ({ page }, name: string) => {
    await closeShareModalIfOpen(page)
    await openFileList(page)
    const tab = page.locator('.file-tab-name', { hasText: name })
    await tab.waitFor({ timeout: 15_000 })
    await tab.click()
    await expect(fileSwitcherTrigger(page)).toContainText(name)
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
    state.activeTabName = name
  },
)

/** Seeds a second browser context's `localStorage` the same way
 * `seedGithubAuth` (storage backend) and "the owner is signed in with
 * GitHub as ..." (Synced Share identity) do for the main page, then opens
 * the given GitHub-backed tab and the share modal's Synced-link tab.
 * `context.addInitScript` (not `page.addInitScript`) so it applies before
 * this context's very first navigation. */
async function loadSecondContextOnGithubFile(
  browser: Browser,
  login: string,
  tabName: string,
): Promise<Page> {
  const context = await browser.newContext()
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  await context.addInitScript(
    ({
      owner,
      login,
      token,
    }: {
      owner: string
      login: string
      token: string
    }) => {
      localStorage.setItem(
        'jianpu:storage-backend:v1',
        JSON.stringify({ backend: 'github', github: { owner } }),
      )
      localStorage.setItem(
        'jianpu:github-auth:v1',
        JSON.stringify({ token: 'fake-token', scopes: ['repo'] }),
      )
      localStorage.setItem(
        'jianpu:synced-share-github-auth:v1',
        JSON.stringify({ token, login }),
      )
    },
    { owner: OWNER, login, token: syncedShareIdentityTokenFor(login) },
  )
  const page = await context.newPage()
  // A separate device seeing "the same real GitHub repo" -- its own
  // in-memory files map, seeded with the same content as the owner's. This
  // tests the *worker's* idempotent-create logic, not GitHub Contents API
  // fidelity, so a second independent mock with matching seed data is fine.
  await mockGithubContentsApi(page, seededFiles)
  await page.goto('/')
  await openFileList(page)
  const tab = page.locator('.file-tab-name', { hasText: tabName })
  await tab.waitFor({ timeout: 15_000 })
  await tab.click()
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  await openSyncedTab(page)
  state.secondContext = context
  state.secondPage = page
  return page
}

/** Same as `loadSecondContextOnGithubFile` but for a local-only file --
 * no storage-backend/GitHub-auth localStorage, no Contents API mock, just
 * the Synced Share identity. */
async function loadSecondContextOnLocalApp(
  browser: Browser,
  login: string,
): Promise<Page> {
  const context = await browser.newContext()
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  await context.addInitScript(
    ({ login, token }: { login: string; token: string }) => {
      localStorage.setItem(
        'jianpu:synced-share-github-auth:v1',
        JSON.stringify({ token, login }),
      )
    },
    { login, token: syncedShareIdentityTokenFor(login) },
  )
  const page = await context.newPage()
  await page.goto('/')
  await openSyncedTab(page)
  state.secondContext = context
  state.secondPage = page
  return page
}

When(
  'a separate browser context loads the same GitHub-backed file, signed in as the same GitHub account {string}',
  async ({ browser }, login: string) => {
    if (!state.activeTabName) {
      throw new Error('no GitHub-backed tab has been selected yet')
    }
    await loadSecondContextOnGithubFile(browser, login, state.activeTabName)
  },
)

When(
  'a separate browser context loads the same GitHub-backed file, signed in as a different GitHub account {string}',
  async ({ browser }, login: string) => {
    if (!state.activeTabName) {
      throw new Error('no GitHub-backed tab has been selected yet')
    }
    await loadSecondContextOnGithubFile(browser, login, state.activeTabName)
  },
)

When(
  'a separate browser context loads the app, signed in as the same GitHub account {string}',
  async ({ browser }, login: string) => {
    await loadSecondContextOnLocalApp(browser, login)
  },
)

When(
  'a separate browser context loads the GitHub-backed file at its renamed path, signed in as the same GitHub account {string}',
  async ({ browser }, login: string) => {
    await loadSecondContextOnGithubFile(browser, login, 'renamed')
  },
)

When(
  'the owner clicks {string} in that separate browser context',
  async ({}, label: string) => {
    expect(label).toBe('Sync')
    const page = state.secondPage
    if (!page) throw new Error('second browser context was not opened yet')
    const startSync = page.getByTestId('share-modal-start-sync')
    if (await startSync.count()) await startSync.click()
    await expect(page.getByTestId('share-modal-copy-synced-link')).toHaveText(
      'Link copied',
    )
    state.secondSyncedShareLink = await page.evaluate(() =>
      navigator.clipboard.readText(),
    )
  },
)

Then(
  'the synced link copied in the separate browser context is identical to the original link',
  async () => {
    if (!sharedState.originalSyncedLink) {
      throw new Error('originalSyncedLink was not captured yet')
    }
    if (!state.secondSyncedShareLink) {
      throw new Error('secondSyncedShareLink was not captured yet')
    }
    expect(state.secondSyncedShareLink).toEqual(sharedState.originalSyncedLink)
  },
)

Then(
  'the synced link copied in the separate browser context is different from the original link',
  async () => {
    if (!sharedState.originalSyncedLink) {
      throw new Error('originalSyncedLink was not captured yet')
    }
    if (!state.secondSyncedShareLink) {
      throw new Error('secondSyncedShareLink was not captured yet')
    }
    expect(state.secondSyncedShareLink).not.toEqual(
      sharedState.originalSyncedLink,
    )
  },
)

Then('the copied sync link is different from the original link', async () => {
  if (!sharedState.originalSyncedLink) {
    throw new Error('originalSyncedLink was not captured yet')
  }
  if (!sharedState.syncedShareLink) {
    throw new Error('syncedShareLink was not captured yet')
  }
  expect(sharedState.syncedShareLink).not.toEqual(
    sharedState.originalSyncedLink,
  )
})

When(
  'the owner renames the active file to {string}',
  async ({ page }, newName: string) => {
    await closeShareModalIfOpen(page)
    await openFileList(page)
    const activeTab = page.locator('.file-tab--active .file-tab-name')
    await activeTab.dblclick()
    const input = page.locator('.file-tab--active input.file-tab-name')
    await input.fill(newName)
    await input.press('Enter')

    // Mirror the rename into `seededFiles` too, so a later "a separate
    // browser context loads the GitHub-backed file at its renamed path ..."
    // step -- a different device fetching a fresh listing -- sees the file
    // at its new path, matching what a real rename (create-at-new-path +
    // delete-at-old-path against the real Contents API) actually leaves on
    // GitHub. The owner's own page's mock already reflects this on its own
    // (its `mockGithubContentsApi` closure was mutated directly by the
    // rename's real PUT/DELETE calls) -- this only updates the snapshot
    // future `mockGithubContentsApi(page, seededFiles)` calls seed from.
    if (state.activeTabName) {
      const oldPath = `scores/${state.activeTabName}.jianpu`
      const content = seededFiles[oldPath]
      if (content !== undefined) {
        delete seededFiles[oldPath]
        seededFiles[`scores/${newName}.jianpu`] = content
      }
    }
    state.activeTabName = newName
  },
)

Then('the share modal still shows the stop-sync button', async ({ page }) => {
  // Reopen the modal -- the rename step above closed it to reach the
  // header's file-switcher trigger (see `closeShareModalIfOpen`'s own
  // comment); the sync itself was never affected by the modal's visibility.
  await openSyncedTab(page)
  await expect(page.getByTestId('share-modal-stop-sync')).toBeVisible()
})

/** Extracts just the share id from a synced-share URL --
 * `buildSyncedShareUrl`'s human-readable filename suffix is cosmetic only
 * (see its own doc comment: "a rename after sharing doesn't invalidate the
 * link, it just makes this cosmetic copy stale"), so "unchanged after the
 * rename" means the id, not the whole URL string. */
function syncedShareId(link: string): string {
  const match = /#synced=([0-9A-Za-z_-]{11})/.exec(link)
  if (!match?.[1]) throw new Error(`not a synced share link: ${link}`)
  return match[1]
}

Then('the synced link is unchanged after the rename', async ({ page }) => {
  if (!sharedState.originalSyncedLink) {
    throw new Error('originalSyncedLink was not captured yet')
  }
  await page.getByTestId('share-modal-copy-synced-link').click()
  const copied = await page.evaluate(() => navigator.clipboard.readText())
  expect(syncedShareId(copied)).toEqual(
    syncedShareId(sharedState.originalSyncedLink),
  )
})
