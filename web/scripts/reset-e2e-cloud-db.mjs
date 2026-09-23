#!/usr/bin/env node
// Wipes the local D1 cloud-backend rows owned by the synthetic e2e accounts
// (`e2e-test-user`, `e2e-test-user-two` -- see `mockGithubIdentity.mjs`)
// before every local e2e run.
//
// Why this exists: unlike CI (`.github/workflows/pages.yml`), which never
// runs this suite at all and wouldn't have this problem anyway since every
// run starts from a throwaway checkout, a local run reuses the same
// `crates/live-share-worker/.wrangler/state` D1 file across every
// invocation (`playwright.config.ts`'s `reuseExistingServer`, plus `just
// dev`/`dekit.yaml` pointing the same `wrangler dev --port 8787` at that
// same file for manual dev browsing). Scenarios that create files under
// app-assigned default names (`files-cloud-backend.feature`'s "untitled"/
// "source 2"-style duplicates) leave real rows behind with no UI path to
// hard-delete them -- `fileStore.ts`'s `reservedNames()` treats binned
// (soft-deleted) names as permanently taken too -- so every successful
// local run permanently reserves one more numbered name
// ("source 2", "source 3", ... eventually "source 12"), and the *next*
// local run's hardcoded "the active tab becomes 'source 2'" expectation
// falls further behind reality until it fails outright. Deleting only the
// two synthetic e2e accounts' rows (never the developer's own real GitHub
// login's files, which live under a different, real `owner_user_id`)
// restores every run to the same starting state CI never had to think
// about.
//
// Hard SQL delete (not the app's soft-delete route) is deliberate and safe
// here: this is throwaway local dev/test data in a gitignored sqlite file
// (see `.gitignore`'s `/crates/live-share-worker/.wrangler`), and the
// synthetic accounts have no "bin" a real user could want restored from.
import { execFileSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const workerDir = fileURLToPath(
  new URL('../../crates/live-share-worker', import.meta.url),
)

// Keep in sync with `e2e/mockGithubIdentity.mjs`'s `KNOWN_GITHUB_USER_IDS`
// -- every login an e2e scenario ever signs in as needs its GitHub id
// listed here too, or its leftover rows won't be cleaned up.
const SYNTHETIC_GITHUB_USER_IDS = [987654321, 987654322, 987654323]

const syntheticOwners = `SELECT user_id FROM user_identities
  WHERE provider = 'github'
    AND provider_user_id IN (${SYNTHETIC_GITHUB_USER_IDS.join(',')})`

// `shares.file_id` references `files(id)`, so a file's share has to go
// before the file itself or D1's foreign-key check rejects the delete.
const sql = `DELETE FROM shares WHERE file_id IN (
  SELECT id FROM files WHERE owner_user_id IN (${syntheticOwners})
);
DELETE FROM files WHERE owner_user_id IN (${syntheticOwners});`

try {
  execFileSync(
    'npx',
    ['wrangler', 'd1', 'execute', 'DB', '--local', '--command', sql],
    { cwd: workerDir, stdio: 'inherit' },
  )
} catch {
  // No local D1 yet (first-ever run -- migrations haven't created `files`/
  // `user_identities` -- or `.wrangler/state` doesn't exist at all): there
  // is nothing to reset, so this is a normal, non-fatal outcome, not an
  // error worth failing the e2e run over.
  console.warn(
    'reset-e2e-cloud-db: no local D1 to reset yet (this is normal on a first run)',
  )
}
