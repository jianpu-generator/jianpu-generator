// Shared GitHub identity table for the Synced Share sign-in mock
// (`mock-github-oauth-server.mjs`'s `GET /user` handler) and the step that
// seeds a signed-in token (`synced-share-button.steps.ts`'s "the owner is
// signed in with GitHub as {string}").
//
// Deliberately a plain `.mjs` with no side effects at import time -- NOT
// imported from `mock-github-oauth-server.mjs`'s own module (nor does this
// module import that one), because that file's top-level `server.listen(...)`
// call would fire a second time if it were ever imported from the Playwright
// test process. Both `mock-github-oauth-server.mjs` (a separate Node process)
// and the Playwright step file import this module independently, so the
// login->id mapping is defined exactly once.
export const DEFAULT_MOCK_GITHUB_LOGIN = 'e2e-test-user'
export const DEFAULT_MOCK_GITHUB_USER_ID = 987654321

// Every login these e2e scenarios sign in as needs a fixed, distinct id
// here -- add one whenever a scenario introduces a new login.
const KNOWN_GITHUB_USER_IDS = {
  [DEFAULT_MOCK_GITHUB_LOGIN]: DEFAULT_MOCK_GITHUB_USER_ID,
  'e2e-test-user-two': 987654322,
}

/** The bearer token `'the owner is signed in with GitHub as {string}'`
 * should seed for a given login. Keep the default login's token byte-for-byte
 * unchanged -- `synced-share-github-signin.feature`'s revoke scenario asserts
 * on the literal string `'e2e-fake-synced-share-token'`. */
export function syncedShareIdentityTokenFor(login) {
  return login === DEFAULT_MOCK_GITHUB_LOGIN
    ? 'e2e-fake-synced-share-token'
    : `e2e-fake-synced-share-token:${login}`
}

/** Reverses `syncedShareIdentityTokenFor` -- the identity `GET /user` should
 * report for a given bearer token. Throws for a login with no known id
 * rather than silently falling back, so a scenario that introduces a new
 * login without registering it here fails loudly instead of resolving to
 * the wrong account. */
export function identityForSyncedShareToken(token) {
  const match = /^e2e-fake-synced-share-token:(.+)$/.exec(token ?? '')
  const login = match ? match[1] : DEFAULT_MOCK_GITHUB_LOGIN
  const id = KNOWN_GITHUB_USER_IDS[login]
  if (id === undefined) {
    throw new Error(
      `mockGithubIdentity: no known GitHub user id for login ${JSON.stringify(login)} -- add one to KNOWN_GITHUB_USER_IDS`,
    )
  }
  return { id, login }
}
