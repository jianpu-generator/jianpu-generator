// The GitHub OAuth client id the e2e worker runs with. `playwright.config.ts`
// passes it to `cf dev` (as `--var`), and `mock-github-oauth-server.ts`
// serves its revocation endpoints under it, so the two agree by construction.
// A side-effect-free module, since the mock server is its own Node process.
export const E2E_GITHUB_CLIENT_ID = 'e2e-test-client-id'
