import {
  DEMO_FILE_NAMES,
  type FileStoreState,
  generateFileId,
} from '../fileStore'
import { isNameTakenError } from './cloudBackendHttp'

/** The single name added to `userFiles` between two `FileStoreState`s --
 * recovers which file a pure `fileStore.ts` transform just created, renamed
 * to, or restored. */
export function addedName(
  before: FileStoreState,
  after: FileStoreState,
): string | undefined {
  return Object.keys(after.userFiles).find(
    (name) => !(name in before.userFiles),
  )
}

/** Recomputes a fresh name for a `409 {code: "name_taken"}` retry --
 * mirrors the increment-suffix half of `fileStore.ts`'s private (not
 * exported) `uniqueName`, duplicated here for the same reason as
 * `addedName` above. Always treats `rejectedName` as taken (the whole point
 * of calling this is that the server just rejected it), in addition to
 * every name reserved in `state` -- so a retry can't reproduce the same
 * collision against the caller's own in-memory names either. */
function nextNameAfterCollision(
  rejectedName: string,
  state: FileStoreState,
): string {
  const taken = new Set([
    ...DEMO_FILE_NAMES,
    ...Object.keys(state.userFiles),
    ...Object.keys(state.bin),
    rejectedName,
  ])
  const dot = rejectedName.lastIndexOf('.')
  const stem = dot > 0 ? rejectedName.slice(0, dot) : rejectedName
  const ext = dot > 0 ? rejectedName.slice(dot) : '.jianpu'
  let n = 2
  while (taken.has(`${stem} ${n}${ext}`)) n++
  return `${stem} ${n}${ext}`
}

/** Substitutes `newName` for `oldName` as a `userFiles`/`fileIds` key,
 * carrying `active` along if it pointed at the renamed key -- applied after
 * a name-collision retry succeeded under a different name than the pure
 * transform originally picked, so the `FileStoreState` this backend returns
 * stays consistent with what the server actually holds. */
export function withRenamedKey(
  state: FileStoreState,
  oldName: string,
  newName: string,
): FileStoreState {
  const content = state.userFiles[oldName]
  if (content === undefined) return state
  const { [oldName]: _removedFile, ...restFiles } = state.userFiles
  const { [oldName]: fileId, ...restIds } = state.fileIds
  return {
    ...state,
    active: state.active === oldName ? newName : state.active,
    userFiles: { ...restFiles, [newName]: content },
    fileIds: { ...restIds, [newName]: fileId ?? generateFileId() },
  }
}

/** Runs `attempt(name)` with `name` first, and on a `409 {code:
 * "name_taken"}` response, retries once with a freshly recomputed unique
 * name -- transparent to the caller (no error surfaced for this specific,
 * rare race). A second collision in a row is not retried again; it
 * propagates to `runOp`'s normal classification, which falls through to
 * `'unknown'` (mirrors this crate's `share_id.rs` philosophy of not looping
 * indefinitely on an astronomically unlikely repeat). Returns the name the
 * attempt actually succeeded under, so the caller can reconcile its
 * `FileStoreState` if it differs from `name`. */
export async function withNameCollisionRetry(
  name: string,
  state: FileStoreState,
  attempt: (name: string) => Promise<unknown>,
): Promise<string> {
  try {
    await attempt(name)
    return name
  } catch (error) {
    if (!isNameTakenError(error)) throw error
    const retryName = nextNameAfterCollision(name, state)
    await attempt(retryName)
    return retryName
  }
}
