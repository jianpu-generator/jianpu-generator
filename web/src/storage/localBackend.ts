import {
  createFile,
  deleteFile,
  deserializeFileStore,
  duplicateFile,
  type FileStoreState,
  importSharedFile,
  readInitialFileStore,
  renameFile,
  restoreFile,
  updateActiveContent,
} from '../fileStore'
import type { SaveStatus, StorageBackend } from './types'

/**
 * Reads the initial `FileStoreState` from `localStorage`. Exposed separately
 * (in addition to being used by `localBackend.load`) because
 * `useStorageBackend` needs a *synchronous* initial value for
 * `useLocalStorage`'s initializer — `StorageBackend.load` is async to
 * accommodate future network-backed backends, but the local backend's
 * underlying read is always instant, so there is no behavior change in
 * seeding the state this way.
 */
export function readInitialStoreSync(): FileStoreState {
  return readInitialFileStore()
}

/**
 * Deserializes a raw `localStorage` value into a `FileStoreState`. Used
 * directly as `useLocalStorage`'s `deserializer` for the same
 * synchronous-seeding reason as `readInitialStoreSync`.
 */
export function deserializeStoreSync(raw: string): FileStoreState {
  return deserializeFileStore(raw)
}

/**
 * Thin async adapter over `fileStore.ts`'s pure, synchronous functions.
 * `saveContent` is a no-op: `useLocalStorage` (used by `useStorageBackend`)
 * already persists to `localStorage` on every state change, so there is
 * nothing left to explicitly save.
 */
export const localBackend: StorageBackend = {
  kind: 'local',

  load: () => Promise.resolve(readInitialStoreSync()),

  createFile: (state) => Promise.resolve(createFile(state)),

  duplicateFile: (state) => Promise.resolve(duplicateFile(state)),

  importFile: (state, filename, content) =>
    Promise.resolve(importSharedFile(state, filename, content)),

  renameFile: (state, from, to) => Promise.resolve(renameFile(state, from, to)),

  deleteFile: (state, name) => Promise.resolve(deleteFile(state, name)),

  restoreFile: (state, name) => Promise.resolve(restoreFile(state, name)),

  updateActiveContent: (state, content) => updateActiveContent(state, content),

  saveContent: () => Promise.resolve(),

  status: (): SaveStatus => 'idle',
}
