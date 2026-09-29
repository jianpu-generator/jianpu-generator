import createClient from 'openapi-fetch'
import type { paths } from '../src/generated/live-share-worker/schema'
import { CLOUD_WORKER_ORIGIN } from './cloudFileHelpers.ts'

// Wipes the local D1 cloud-backend rows owned by the synthetic e2e accounts
// (`KNOWN_GITHUB_USER_IDS` in `mockGithubIdentity.ts`) before every run.
//
// Why this exists: CI never runs this suite, but a local run reuses the
// same `crates/live-share-worker/.wrangler/e2e-state` D1 dir across every
// invocation (`playwright.config.ts`'s `reuseExistingServer`). Scenarios
// that create files under app-assigned default names
// (`files-cloud-backend.feature`'s "untitled"/"source 2"-style duplicates)
// leave real rows behind with no UI path to hard-delete them --
// `fileStore.ts`'s `reservedNames()` treats binned names as taken too -- so
// every run would reserve one more numbered name ("source 2", "source 3",
// ...) and the next run's "the active tab becomes 'source 2'" expectation
// would fall further behind until it fails outright.
//
// The delete itself is the worker's test-only `POST /e2e/reset` route
// (`crates/live-share-worker/src/e2e_reset.rs`), which owns the SQL and
// only touches the accounts `playwright.config.ts` allowlists for it -- so
// nothing here knows the D1 schema, and any failure (including the route
// being missing because a worker started without that config is being
// reused) fails the run instead of silently resetting nothing.
export default async function globalSetup() {
  const worker = createClient<paths>({ baseUrl: CLOUD_WORKER_ORIGIN })
  const { data, error, response } = await worker.POST('/e2e/reset', {
    body: {},
  })
  if (!data) {
    const hint =
      response.status === 404
        ? ' -- the worker on this port was started without the e2e reset var; stop it so Playwright starts its own'
        : ''
    throw new Error(
      `e2e reset failed with status ${response.status}${hint}: ${JSON.stringify(error)}`,
    )
  }
  console.log(
    `e2e reset: deleted ${data.deletedFiles} files and ${data.deletedShares} shares`,
  )
}
