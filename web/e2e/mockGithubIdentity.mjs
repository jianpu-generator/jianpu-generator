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
  // Dedicated to `file-op-error-cloud.feature`'s failed-create scenario --
  // that scenario asserts no file named exactly "untitled" exists for the
  // signed-in account after a failed create, but the app-assigned default
  // name for every "New" click is that same literal "untitled" regardless
  // of which scenario clicked it. Sharing `DEFAULT_MOCK_GITHUB_LOGIN` with
  // `files-cloud-backend.feature`'s "Creating a file persists it to the
  // cloud backend" scenario (which legitimately creates a file named
  // "untitled" via the same button) is a real cross-scenario race once
  // `fullyParallel` runs both in different workers at once: whichever
  // scenario's "untitled" file lands first is visible to the other's
  // assertion too, since both would otherwise share one cloud account.
  'e2e-test-user-create-error': 987654323,
  // Dedicated to `synced-share-github-signin.feature`'s multi-device
  // sign-out scenario -- revoking this account's GitHub grant revokes every
  // token the mock server minted for it (see `mock-github-oauth-server.mjs`),
  // so it must never be shared with a scenario running in parallel.
  'e2e-test-user-multi-device': 987654324,
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

/** The fixed GitHub identity for `login`. Throws for a login with no known
 * id rather than silently falling back, so a scenario that introduces a new
 * login without registering it here fails loudly instead of resolving to
 * the wrong account. */
export function identityForLogin(login) {
  const id = KNOWN_GITHUB_USER_IDS[login]
  if (id === undefined) {
    throw new Error(
      `mockGithubIdentity: no known GitHub user id for login ${JSON.stringify(login)} -- add one to KNOWN_GITHUB_USER_IDS`,
    )
  }
  return { id, login }
}

const AUTHORIZATION_CODE_PREFIX = 'e2e-fake-authorization-code-'
const AUTHORIZATION_CODE_LOGIN_SEPARATOR = ':as:'

/** A fresh single-use authorization code for the mocked GitHub popup to
 * hand back. With `login`, the mock server's token exchange mints a
 * distinct, revocable token for that account (`loginFromAuthorizationCode`)
 * -- the way real GitHub's code identifies who approved the request.
 * Without it, the exchange returns the legacy shared token for the default
 * login, outside the mock's revocation tracking. */
export function authorizationCodeFor(login) {
  const code = `${AUTHORIZATION_CODE_PREFIX}${crypto.randomUUID()}`
  return login ? `${code}${AUTHORIZATION_CODE_LOGIN_SEPARATOR}${login}` : code
}

/** Reverses `authorizationCodeFor`: the login a code was issued for, or
 * `null` for a login-less code. */
export function loginFromAuthorizationCode(code) {
  const index = code.indexOf(AUTHORIZATION_CODE_LOGIN_SEPARATOR)
  return index === -1
    ? null
    : code.slice(index + AUTHORIZATION_CODE_LOGIN_SEPARATOR.length)
}

/** Reverses `syncedShareIdentityTokenFor` -- the identity `GET /user` should
 * report for a given bearer token (see `identityForLogin`). */
export function identityForSyncedShareToken(token) {
  const match = /^e2e-fake-synced-share-token:(.+)$/.exec(token ?? '')
  return identityForLogin(match ? match[1] : DEFAULT_MOCK_GITHUB_LOGIN)
}
