import type { Route } from '@playwright/test'
import createClient from 'openapi-fetch'
import type {
  components,
  paths,
} from '../src/generated/live-share-worker/schema'
import { syncedShareIdentityTokenFor } from './mockGithubIdentity.mjs'

export type WorkerSchemas = components['schemas']
type PublicFile = WorkerSchemas['PublicFile']
type ApiError = WorkerSchemas['ApiError']
type WorkerPath = keyof paths

// Seeds/mutates real rows in the local `wrangler dev` worker's D1 database
// (see `playwright.config.ts`'s `webServer` array) by calling its `/files/*`
// routes directly from Node -- the e2e equivalent of "this file already
// existed in the cloud before the page under test ever loaded". Unlike the
// deleted `github-contents-mock.ts`, there is no `page.route()` interception
// here at all: `/files/*` is entirely worker+D1 mediated, so calling the
// real worker (through the same generated client the app uses) is both
// simpler and more faithful. Also home to the helpers every step file
// uses to intercept a worker route (`workerRouteGlob`/`matchWorkerRoute`)
// or fake its failure (`fulfillWithApiError`), so route paths and error
// bodies are always the generated ones, never hand-typed.

/** Same origin `playwright.config.ts` points the app's own `VITE_SYNCED_SHARE_HOST`
 * at -- see that file's `webServer` entry for `wrangler dev --port 8797`
 * (deliberately not `just dev`'s 8787, see that entry's comment). */
export const CLOUD_WORKER_ORIGIN = 'http://localhost:8797'

const worker = createClient<paths>({ baseUrl: CLOUD_WORKER_ORIGIN })

/** A `page.route` glob for one of the worker's routes, each `{param}` as
 * `*`. */
export function workerRouteGlob(path: WorkerPath): string {
  return `${CLOUD_WORKER_ORIGIN}${path.replace(/\{[^}]+\}/g, '*')}`
}

/** `url`'s path params if it's a request to `path`, else `null`. */
export function matchWorkerRoute(
  path: WorkerPath,
  url: string,
): Record<string, string> | null {
  const names = [...path.matchAll(/\{([^}]+)\}/g)].map((match) => match[1])
  const pattern = new RegExp(
    `^${CLOUD_WORKER_ORIGIN}${path.replace(/\{[^}]+\}/g, '([^/]+)')}$`,
  )
  const values = pattern.exec(new URL(url).href.replace(/\?.*$/, ''))
  if (!values) return null
  return Object.fromEntries(
    names.map((name, index) => [name, values[index + 1] ?? '']),
  )
}

/** Fakes a worker failure, its body checked against the generated
 * `ApiError` union. */
export async function fulfillWithApiError(
  route: Route,
  status: number,
  error: ApiError,
): Promise<void> {
  await route.fulfill({ status, json: error })
}

/** Unwraps a worker call made from Node, failing the step loudly on any
 * error response. */
async function expectOk<Data>(
  what: string,
  pending: Promise<{ data?: Data; error?: unknown; response: Response }>,
): Promise<Data> {
  const { data, error, response } = await pending
  if (!response.ok) {
    throw new Error(
      `cloudFileHelpers: ${what} failed with ${response.status}: ${JSON.stringify(error)}`,
    )
  }
  return data as Data
}

/** Deterministic default seed content -- distinct enough to tell apart from
 * a scenario's own edits in a failing assertion's diff, but never itself
 * asserted on by name. */
export const DEFAULT_CLOUD_FILE_CONTENT = [
  '# metadata',
  'title = "Cloud Backend Test"',
  '',
  '# parts',
  'Melody [M] = notes',
  '',
  '# score',
  '(bpm=120 key=C4 time=4/4)',
  '1 2 3 4',
].join('\n')

/** Creates a new active cloud file directly against the real local worker,
 * bypassing the browser entirely. `id` mirrors `web/src/fileStore.ts`'s
 * `generateFileId()` (a plain `crypto.randomUUID()`) -- the server trusts
 * the client-generated id verbatim (see `CreateFileRequest`'s doc comment
 * in `protocol.rs`), so any unique string works here too.
 *
 * Idempotent across reruns of the same scenario: `scripts/resolve-e2e-
 * flakes.mjs` reruns a still-failing scenario against the very same
 * already-running `wrangler dev` + local D1 instance (see
 * `playwright.config.ts`'s `reuseExistingServer`), so a plain `POST /files`
 * would fail with `name_taken` on any rerun once the first attempt's row is
 * already sitting there under this exact name -- see `reseedExistingFile`. */
export async function seedCloudFile(
  login: string,
  name: string,
  content: string = DEFAULT_CLOUD_FILE_CONTENT,
): Promise<PublicFile> {
  const identityToken = syncedShareIdentityTokenFor(login)
  const { data, error, response } = await worker.POST('/files', {
    body: { identityToken, id: crypto.randomUUID(), name, content },
  })
  if (data) return data
  if (error?.code === 'name_taken') {
    return await reseedExistingFile(login, name, content)
  }
  throw new Error(
    `cloudFileHelpers: POST /files failed with ${response.status}: ${JSON.stringify(error)}`,
  )
}

/** Reuses a pre-existing row already sitting under `name` for this account
 * (restoring it out of the bin first if a previous run's scenario trashed
 * it) and overwrites its content, so a colliding `seedCloudFile` call ends
 * up in exactly the state a fresh create would have produced. */
async function reseedExistingFile(
  login: string,
  name: string,
  content: string,
): Promise<PublicFile> {
  const identityToken = syncedShareIdentityTokenFor(login)
  const listed = await expectOk(
    'POST /files/list',
    worker.POST('/files/list', { body: { identityToken } }),
  )
  const existing = listed.files.find((file) => file.name === name)
  if (!existing) {
    throw new Error(
      `cloudFileHelpers: seedCloudFile got name_taken for ${JSON.stringify(name)} but /files/list has no matching row`,
    )
  }
  if (existing.trashedAt !== null) {
    await expectOk(
      'POST /files/{id}/restore',
      worker.POST('/files/{id}/restore', {
        params: { path: { id: existing.id } },
        body: { identityToken, name },
      }),
    )
  }
  const updated = await expectOk(
    'POST /files/{id}/content',
    worker.POST('/files/{id}/content', {
      params: { path: { id: existing.id } },
      body: { identityToken, content, expectedRevision: existing.revision },
    }),
  )
  return {
    id: existing.id,
    name,
    content,
    revision: updated.revision,
    trashedAt: null,
  }
}

/** Moves a previously seeded file to the bin, by its real D1 id. */
export async function trashCloudFile(login: string, id: string): Promise<void> {
  await expectOk(
    'POST /files/{id}/delete',
    worker.POST('/files/{id}/delete', {
      params: { path: { id } },
      body: { identityToken: syncedShareIdentityTokenFor(login) },
    }),
  )
}

/** Restores a previously trashed file out of the bin under `name`, by its
 * real D1 id. */
export async function restoreCloudFile(
  login: string,
  id: string,
  name: string,
): Promise<void> {
  await expectOk(
    'POST /files/{id}/restore',
    worker.POST('/files/{id}/restore', {
      params: { path: { id } },
      body: { identityToken: syncedShareIdentityTokenFor(login), name },
    }),
  )
}

/** Convenience: seeds a file and immediately trashes it, for scenarios that
 * need a file to load straight into the bin with no prior UI delete step. */
export async function seedTrashedCloudFile(
  login: string,
  name: string,
  content: string = DEFAULT_CLOUD_FILE_CONTENT,
): Promise<PublicFile> {
  const file = await seedCloudFile(login, name, content)
  await trashCloudFile(login, file.id)
  return file
}

/** Overwrites a file's content directly against the real worker, gated on
 * `expectedRevision` (the same optimistic-concurrency CAS every real save
 * goes through -- see `crate::files::classify_content_write`). Used to
 * simulate "the remote content changed since the browser last saw it" for
 * the conflict-resolution scenarios' "discard mine" path. */
export async function updateCloudFileContent(
  login: string,
  id: string,
  content: string,
  expectedRevision: number,
): Promise<{ revision: number }> {
  return await expectOk(
    'POST /files/{id}/content',
    worker.POST('/files/{id}/content', {
      params: { path: { id } },
      body: {
        identityToken: syncedShareIdentityTokenFor(login),
        content,
        expectedRevision,
      },
    }),
  )
}

/** Fetches a share the way an anonymous viewer would. */
export async function fetchSyncedDoc(
  shareId: string,
): Promise<WorkerSchemas['SyncedDoc']> {
  return await expectOk(
    'GET /shares/{share_id}',
    worker.GET('/shares/{share_id}', {
      params: { path: { share_id: shareId } },
    }),
  )
}
