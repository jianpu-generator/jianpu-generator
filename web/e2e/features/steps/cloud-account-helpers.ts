import type { Page } from '@playwright/test'
import { DEFAULT_MOCK_GITHUB_LOGIN } from '../../mockGithubIdentity.mjs'

// Shared, non-glue helpers for the cloud (D1) storage backend's e2e
// coverage -- imported by several `.steps.ts` files across
// `account-sign-in.feature`, `files-cloud-backend.feature`,
// `cloud-backend-conflict-resolution.feature`, and the `-cloud`
// transpositions of the deleted `-github` feature files. Kept separate from
// `cloud-account.steps.ts` (which owns the actual `Given`/`When`/`Then`
// registrations) so plain functions can be imported without pulling in
// step-registration side effects.

export { DEFAULT_MOCK_GITHUB_LOGIN }

/** Seeds the `'cloud'` storage-backend preference (see
 * `useStorageBackend.ts`'s `StorageBackendPreference`) via `addInitScript`
 * before navigating, then loads the app. Signing in alone
 * (`accountAuth.ts`) never switches the active backend -- see
 * `useStorageBackend.ts`'s own doc comment on that being an explicit,
 * independent choice -- so every scenario that wants to load straight onto
 * the cloud-backed file list (rather than exercising the sign-in ->
 * "select Cloud storage" UI flow itself, see `account-sign-in.feature`)
 * needs to seed this preference directly, same as the deleted GitHub
 * backend's tests seeded `{backend: 'github', github: {owner}}`. */
export async function gotoCloudApp(page: Page): Promise<void> {
  await page.addInitScript(() => {
    localStorage.setItem(
      'jianpu:storage-backend:v1',
      JSON.stringify({ backend: 'cloud' }),
    )
  })
  await page.goto('/')
}
