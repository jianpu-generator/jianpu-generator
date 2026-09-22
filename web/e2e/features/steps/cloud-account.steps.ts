import {
  DEFAULT_CLOUD_FILE_CONTENT,
  seedCloudFile,
} from '../../cloudFileHelpers'
import {
  DEFAULT_MOCK_GITHUB_LOGIN,
  syncedShareIdentityTokenFor,
} from '../../mockGithubIdentity.mjs'
import { BeforeScenario, Given } from './fixtures'

// Shared account sign-in + cloud-file-seeding `Given` steps, used as the
// `Background` for `account-sign-in.feature`, `files-cloud-backend.feature`,
// `cloud-backend-conflict-resolution.feature`, and every `-cloud`
// transposition of a deleted `-github` feature file. Kept in its own file
// (rather than folded into e.g. `files-cloud-backend.steps.ts`) since it's
// shared across so many feature files -- mirrors this project's existing
// convention of a dedicated steps file per reusable concern (compare
// `synced-share-button.steps.ts`'s "the owner is signed in with GitHub as
// {string}").
//
// Module-level, not a fixture -- matches every other `.steps.ts` file's
// established pattern (e.g. `lastSignInPopup` in
// `synced-share-github-signin.steps.ts`). Tracks whichever login the
// current scenario's "an account is signed in as ..." step used, so a
// later "... is seeded for the signed-in account" step in the same
// scenario seeds under the same identity without every feature file having
// to repeat the login in both steps.
let signedInLogin: string = DEFAULT_MOCK_GITHUB_LOGIN

// The most recently seeded file's *display* name (extension stripped),
// tracked the same way as `signedInLogin` above. Needed because the shared
// `e2e-test-user` account is real and persistent across this whole suite
// run, not a fresh in-memory mock scoped to one scenario like the deleted
// GitHub backend's tests had -- by the time a scenario's own "the active
// file" step runs, the account can already hold many *other* scenarios'
// files too (concurrent workers, or leftover rows from earlier runs), so
// grabbing "the first tab in the list" can no longer be assumed to mean
// "the file this scenario just seeded". Steps that need to act on "the
// file I just seeded" (rename/delete/duplicate in
// `files-cloud-backend.steps.ts`) select it by this exact tracked name
// instead.
let lastSeededFileDisplayName = ''

BeforeScenario(async () => {
  signedInLogin = DEFAULT_MOCK_GITHUB_LOGIN
  lastSeededFileDisplayName = ''
})

/**
 * Pre-seeds the unified account sign-in (`accountAuth.ts`) -- bypassing the
 * real popup OAuth round trip, which has its own dedicated coverage in
 * `synced-share-github-signin.feature` and this feature's own "Signing in
 * via the popup ..." scenario. The literal `localStorage` key is
 * deliberately hardcoded here rather than imported from `accountAuth.ts`,
 * matching `synced-share-button.steps.ts`'s "the owner is signed in with
 * GitHub as {string}" -- that key is a stable, deliberately-preserved
 * literal (see `accountAuth.ts`'s own doc comment), not an implementation
 * detail that would justify pulling the app module (and its `usehooks-ts`
 * import chain) into the Playwright test process.
 */
Given(
  'an account is signed in as {string}',
  async ({ page }, login: string) => {
    signedInLogin = login
    await page.addInitScript(
      ({ login, token }: { login: string; token: string }) => {
        localStorage.setItem(
          'jianpu:synced-share-github-auth:v1',
          JSON.stringify({ token, login }),
        )
      },
      { login, token: syncedShareIdentityTokenFor(login) },
    )
  },
)

function trackSeededFileDisplayName(name: string): void {
  lastSeededFileDisplayName = name.replace(/\.jianpu$/, '')
}

Given(
  'a cloud file named {string} is seeded for the signed-in account',
  async ({}, name: string) => {
    await seedCloudFile(signedInLogin, name)
    trackSeededFileDisplayName(name)
  },
)

Given(
  'a cloud file named {string} with content {string} is seeded for the signed-in account',
  async ({}, name: string, content: string) => {
    await seedCloudFile(signedInLogin, name, content)
    trackSeededFileDisplayName(name)
  },
)

/** The login the current scenario's "an account is signed in as ..." step
 * captured -- exported for other `.steps.ts` files that need to seed
 * additional cloud files inline within their own `Given` (e.g. the restore-
 * collision scenario's two-file setup) rather than via the two generic
 * steps above. */
export function currentSignedInLogin(): string {
  return signedInLogin
}

/** The display name (extension stripped) of the most recently seeded file
 * -- see `lastSeededFileDisplayName`'s doc comment above for why steps that
 * act on "the active file" need to select it by this exact name rather
 * than assuming it's whichever tab happens to sort first. */
export function currentSeededFileDisplayName(): string {
  return lastSeededFileDisplayName
}

export { DEFAULT_CLOUD_FILE_CONTENT }
