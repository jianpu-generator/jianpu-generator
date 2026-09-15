import { expect, type Page } from '@playwright/test'
import { Given, Then, When } from './fixtures'
import { openSyncedTab } from './synced-share-button.steps'

// Covers the Synced Share GitHub sign-in UI itself
// (synced-share-github-signin.feature) -- the popup OAuth round trip, the
// "Synced as @username" identity row, and the full-screen verification
// failure dialog. Kept in its own steps file (not
// synced-share-button.steps.ts) since these steps mock the sign-in/worker
// network layer directly rather than reusing a pre-seeded connection.

// The popup navigates to GitHub's real authorization endpoint
// (`GITHUB_AUTHORIZATION_ENDPOINT` in `syncedShareGithubAuthPopup.ts`) -- a
// top-level browser navigation, so `context.route` (not `page.route`, which
// only covers the opener's own page) can intercept it. Fulfilling with a
// redirect straight back to the app's own callback URL (echoing the real
// `state` GitHub would have been asked to echo back) simulates the user
// approving the request without ever making a real GitHub network call.
// The callback page's own exchange (`POST /auth/github/callback`) still
// hits the real local Synced Share worker, which is itself pointed at
// `e2e/mock-github-oauth-server.mjs` (see `playwright.config.ts`) for the
// token exchange and `GET /user` calls that happen worker-side.

// Captured by "the owner clicks \"Sign in with GitHub\" in the prompt" below
// and asserted against by "the GitHub sign-in popup has closed itself" --
// regression coverage for a bug where `SyncedShareGithubCallbackPage`'s
// `window.close()` call is silently refused by the browser once the popup
// has navigated across origins more than once (about:blank -> GitHub's
// authorization endpoint -> back to this app's own callback page), leaving
// the popup sitting on "Signed in. You can close this window." forever
// instead of closing itself as the user expects. Module-level since each
// scenario runs in its own worker process, matching the established
// `let ... = ...` capture pattern used across the other `.steps.ts` files
// (e.g. `putBodies` in `autosave-github.steps.ts`).
let lastSignInPopup: Page | null = null

Given(
  'the GitHub authorization popup is mocked to redirect back successfully',
  async ({ context }) => {
    await context.route(
      'https://github.com/login/oauth/authorize**',
      async (route) => {
        const requestUrl = new URL(route.request().url())
        const state = requestUrl.searchParams.get('state') ?? ''
        const redirectUri = requestUrl.searchParams.get('redirect_uri')
        if (!redirectUri) {
          await route.abort()
          return
        }
        const target = new URL(redirectUri)
        // Unique per attempt, not a fixed string: the mock token-exchange
        // server (`mock-github-oauth-server.mjs`) enforces real GitHub's
        // single-use-code behavior, and that server process (and its
        // used-codes set) persists across the whole suite run -- a fixed
        // code would make one scenario's exchange poison every other
        // scenario's (or retry's) use of this same route.
        target.searchParams.set(
          'code',
          `e2e-fake-authorization-code-${crypto.randomUUID()}`,
        )
        target.searchParams.set('state', state)
        await route.fulfill({
          status: 302,
          headers: { Location: target.toString() },
        })
      },
    )
  },
)

// Simulates the browser refusing to open the popup at all
// (`window.open` returning `null`) -- `openSyncedShareGithubSignInPopup`
// resolves this as a `'blocked'` failure without ever reaching GitHub.
Given(
  'the GitHub sign-in popup is blocked by the browser',
  async ({ page }) => {
    await page.addInitScript(() => {
      window.open = () => null
    })
  },
)

Then('the sign-in prompt is shown', async ({ page }) => {
  await expect(page.getByTestId('share-modal-signin-state')).toBeVisible()
})

When(
  'the owner clicks "Sign in with GitHub" in the prompt',
  async ({ page, context }) => {
    // Not every scenario using this shared step actually gets a popup --
    // "A blocked GitHub sign-in popup returns to idle with no share
    // created" stubs `window.open` to return `null` (see "the GitHub
    // sign-in popup is blocked by the browser" above), so no `'page'`
    // event ever fires. `.catch(() => null)` keeps that scenario from
    // hanging on `context.waitForEvent('page')` for the full test timeout;
    // `lastSignInPopup` staying `null` is fine there since that scenario
    // never asserts against it.
    const [popup] = await Promise.all([
      context.waitForEvent('page', { timeout: 2_000 }).catch(() => null),
      page.getByTestId('share-modal-sign-in-with-github').click(),
    ])
    lastSignInPopup = popup
  },
)

Then(
  'the identity row shows signed in as {string}',
  async ({ page }, login: string) => {
    await expect(page.getByTestId('share-modal-identity')).toContainText(
      `Signed in as @${login}`,
    )
  },
)

Then('the GitHub sign-in popup has closed itself', async () => {
  if (!lastSignInPopup) {
    throw new Error(
      'no GitHub sign-in popup was captured -- did "the owner clicks \\"Sign in with GitHub\\" in the prompt" run first?',
    )
  }
  const popup = lastSignInPopup
  if (!popup.isClosed()) {
    await popup.waitForEvent('close', { timeout: 5_000 }).catch(() => {})
  }
  expect(popup.isClosed()).toBe(true)
})

// Regression coverage for a bug where a `200 OK` response from the worker's
// `POST /auth/github/callback` with a body that couldn't be parsed as JSON
// (e.g. a truncated response mid-restart) threw out of `response.json()`
// uncaught inside the popup, and that rejection went unhandled -- the popup
// never reached the code that flips its status and calls `window.close()`,
// so it sat on "Signing in with GitHub…" forever and the opener's own button
// never left its "Signing in…" state either (see
// `syncedShareGithubAuthCallback.ts`/`SyncedShareGithubCallbackPage.tsx`). This
// fetch runs inside the popup page, a separate `Page` from the opener within
// the same `BrowserContext`, so `context.route` (not `page.route`) is what
// catches it -- same reasoning as "the GitHub authorization popup is mocked
// to redirect back successfully" above.
Given(
  'the Synced Share worker returns an unparseable body from the next GitHub token exchange',
  async ({ context }) => {
    await context.route(
      'http://localhost:8787/auth/github/callback',
      async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: 'not json',
        })
      },
    )
  },
)

Then(
  'the sign-in prompt shows a sign-in error containing {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('share-modal-signin-error')).toContainText(
      text,
    )
  },
)

Then(
  'the "Sign in with GitHub" button is no longer stuck signing in',
  async ({ page }) => {
    const button = page.getByTestId('share-modal-sign-in-with-github')
    await expect(button).toHaveText('Sign in with GitHub')
    await expect(button).toBeEnabled()
  },
)

Then(
  'the synced share identity row reads {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('share-modal-identity')).toContainText(text)
  },
)

When('the owner clicks "Log out" in the identity row', async ({ page }) => {
  await page.getByTestId('share-modal-logout').click()
})

// This fetch (`revokeSyncedShareGithubGrant`, fired from `disconnectGithub`
// in `useSyncedShareOwner.ts`) runs on the opener page itself, not inside a
// popup -- so `page.route`, not `context.route`, is what catches it (same
// reasoning as the create-share mock further down this file). Module-level
// capture, matching this file's established pattern (e.g.
// `lastSignInPopup` above).
let lastRevokeRequestBody: { identityToken?: string } | null = null

Given('the GitHub grant-revocation endpoint is mocked', async ({ page }) => {
  await page.route(
    'http://localhost:8787/auth/github/revoke',
    async (route) => {
      lastRevokeRequestBody = route.request().postDataJSON()
      await route.fulfill({ status: 200, body: '' })
    },
  )
})

Then(
  'the worker was asked to revoke the GitHub grant for {string}',
  async ({}, token: string) => {
    expect(lastRevokeRequestBody?.identityToken).toBe(token)
  },
)

// The owner's browser-side fetch straight to the Synced Share worker's
// `POST /shares` (see `useSyncedShareOwner.ts`'s `createShare`) -- unlike
// the worker's own outbound GitHub calls (mocked via a real local server,
// see `synced-share-github-signin.feature`'s doc comment), this one runs in
// the browser itself, so `page.route` can intercept it directly without
// needing the real worker involved at all.
Given(
  'the Synced Share worker rejects the next create-share request with a verification failure',
  async ({ page }) => {
    await page.route('http://localhost:8787/shares', async (route) => {
      if (route.request().method() !== 'POST') {
        await route.continue()
        return
      }
      await route.fulfill({
        status: 401,
        contentType: 'application/json',
        body: JSON.stringify({
          reason: 'GitHub verification failed',
          failedAt: Date.now(),
          attempts: 2,
        }),
      })
    })
  },
)

Then(
  'the synced share error dialog is shown with a {string} link',
  async ({ page }, linkText: string) => {
    await expect(page.getByTestId('synced-share-error-dialog')).toBeVisible()
    await expect(
      page.getByTestId('synced-share-error-dialog-issue-link'),
    ).toHaveText(linkText)
  },
)

Then(
  'the synced share error dialog shows the reason {string}',
  async ({ page }, reason: string) => {
    await expect(
      page.getByTestId('synced-share-error-dialog-details'),
    ).toContainText(reason)
  },
)

When('the owner dismisses the synced share error dialog', async ({ page }) => {
  await page.getByTestId('synced-share-error-dialog-close').click()
})

Then('the synced share error dialog is gone', async ({ page }) => {
  await expect(page.getByTestId('synced-share-error-dialog')).toHaveCount(0)
})

// The error dialog closes the share modal as a side effect of opening --
// two independent Radix `Dialog.Root`s open at once fight over the focus
// trap, and the error dialog wins (see `openSyncedTab`'s doc comment in
// `synced-share-button.steps.ts`) -- so scenarios that need to inspect the
// modal's state afterwards must explicitly re-open it first.
When('the owner reopens the share modal', async ({ page }) => {
  await openSyncedTab(page)
})
