import { type FileStoreState, generateFileId } from '../fileStore'

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

/** A fresh name for a queued `CreateFile`/`RenameFile`/`RestoreFile` whose
 * `rejectedName` the server reported as taken, when no `FileStoreState` is
 * at hand (the outbox delivers long after the edit). Increments a trailing
 * ` N` before the extension, so a second collision yields ` 3`, not ` 2 2`. */
export function suggestNameAfterCollision(rejectedName: string): string {
  const dot = rejectedName.lastIndexOf('.')
  const stem = dot > 0 ? rejectedName.slice(0, dot) : rejectedName
  const ext = dot > 0 ? rejectedName.slice(dot) : '.jianpu'
  const numbered = /^(.*) (\d+)$/.exec(stem)
  return numbered
    ? `${numbered[1]} ${Number(numbered[2]) + 1}${ext}`
    : `${stem} 2${ext}`
}
