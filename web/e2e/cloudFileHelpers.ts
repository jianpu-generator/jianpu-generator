import { syncedShareIdentityTokenFor } from './mockGithubIdentity.mjs'

// Seeds/mutates real rows in the local `wrangler dev` worker's D1 database
// (see `playwright.config.ts`'s `webServer` array) by calling its `/files/*`
// routes directly from Node -- the e2e equivalent of "this file already
// existed in the cloud before the page under test ever loaded". Unlike the
// deleted `github-contents-mock.ts`, there is no `page.route()` interception
// here at all: `/files/*` is entirely worker+D1 mediated, so a plain `fetch`
// against the real worker is both simpler and more faithful (see
// `crates/live-share-worker/src/handlers.rs`'s actual route shapes, which
// this file's request bodies mirror exactly).

/** Same origin `playwright.config.ts` points the app's own `VITE_SYNCED_SHARE_HOST`
 * at -- see that file's `webServer` entry for `wrangler dev --port 8787`. */
export const CLOUD_WORKER_ORIGIN = 'http://localhost:8787'

/** Wire shape of `crate::files::PublicFile` (`camelCase`), same shape
 * `cloudBackend.ts`'s own `PublicFileWire` mirrors. */
export interface CloudFileWire {
  id: string
  name: string
  content: string
  revision: number
  trashedAt: number | null
}

function postToWorkerRaw(path: string, body: object): Promise<Response> {
  return fetch(`${CLOUD_WORKER_ORIGIN}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
}

async function postToWorker(path: string, body: object): Promise<unknown> {
  const response = await postToWorkerRaw(path, body)
  if (response.status === 204) return null
  if (!response.ok) {
    const text = await response.text().catch(() => '')
    throw new Error(
      `cloudFileHelpers: POST ${path} failed with ${response.status}: ${text}`,
    )
  }
  return await response.json()
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
 * would 409 (`name_taken`) on any rerun once the first attempt's row is
 * already sitting there under this exact name -- see `reseedExistingFile`. */
export async function seedCloudFile(
  login: string,
  name: string,
  content: string = DEFAULT_CLOUD_FILE_CONTENT,
): Promise<CloudFileWire> {
  const identityToken = syncedShareIdentityTokenFor(login)
  const response = await postToWorkerRaw('/files', {
    identityToken,
    id: crypto.randomUUID(),
    name,
    content,
  })
  if (response.ok) {
    return (await response.json()) as CloudFileWire
  }
  if (response.status === 409) {
    return await reseedExistingFile(login, name, content)
  }
  const text = await response.text().catch(() => '')
  throw new Error(
    `cloudFileHelpers: POST /files failed with ${response.status}: ${text}`,
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
): Promise<CloudFileWire> {
  const identityToken = syncedShareIdentityTokenFor(login)
  const listed = (await postToWorker('/files/list', { identityToken })) as {
    files: CloudFileWire[]
  }
  const existing = listed.files.find((file) => file.name === name)
  if (!existing) {
    throw new Error(
      `cloudFileHelpers: seedCloudFile got a 409 for ${JSON.stringify(name)} but /files/list has no matching row`,
    )
  }
  if (existing.trashedAt !== null) {
    await postToWorker(`/files/${existing.id}/restore`, {
      identityToken,
      name,
    })
  }
  const updated = (await postToWorker(`/files/${existing.id}/content`, {
    identityToken,
    content,
    expectedRevision: existing.revision,
  })) as { revision: number }
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
  await postToWorker(`/files/${id}/delete`, {
    identityToken: syncedShareIdentityTokenFor(login),
  })
}

/** Restores a previously trashed file out of the bin under `name`, by its
 * real D1 id. */
export async function restoreCloudFile(
  login: string,
  id: string,
  name: string,
): Promise<void> {
  await postToWorker(`/files/${id}/restore`, {
    identityToken: syncedShareIdentityTokenFor(login),
    name,
  })
}

/** Convenience: seeds a file and immediately trashes it, for scenarios that
 * need a file to load straight into the bin with no prior UI delete step. */
export async function seedTrashedCloudFile(
  login: string,
  name: string,
  content: string = DEFAULT_CLOUD_FILE_CONTENT,
): Promise<CloudFileWire> {
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
  const result = (await postToWorker(`/files/${id}/content`, {
    identityToken: syncedShareIdentityTokenFor(login),
    content,
    expectedRevision,
  })) as { revision: number } | null
  if (!result)
    throw new Error('updateCloudFileContent: worker returned no body')
  return result
}
