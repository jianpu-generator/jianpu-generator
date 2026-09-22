import type { FileStoreState } from '../fileStore'

/**
 * Persistence/save-indicator state exposed by a backend. `'offline'` and
 * `'error'` are only reachable once network-backed backends (e.g. cloud)
 * exist; the local backend is always `'idle'`.
 */
export type SaveStatus = 'idle' | 'saving' | 'saved' | 'error' | 'offline'

/**
 * A pluggable storage backend for the file store. The browser
 * `localStorage` backend and the D1-backed `'cloud'` backend
 * (`cloudBackend.ts`) both implement this interface, so `FileStoreState`
 * mutations are backend-agnostic from the hook/UI layer's perspective.
 *
 * `updateActiveContent` is a plain synchronous state setter with no
 * persistence side effect; `saveContent` is the explicit call that persists
 * the active file's content. Debounce ownership lives in the hook layer
 * (`useStorageBackend`), not here, to avoid two independent debounce
 * mechanisms with unclear ownership.
 */
export interface StorageBackend {
  readonly kind: 'local' | 'cloud'
  load(): Promise<FileStoreState>
  createFile(state: FileStoreState): Promise<FileStoreState>
  duplicateFile(state: FileStoreState): Promise<FileStoreState>
  importFile(
    state: FileStoreState,
    filename: string,
    content: string,
  ): Promise<FileStoreState>
  renameFile(
    state: FileStoreState,
    from: string,
    to: string,
  ): Promise<FileStoreState>
  deleteFile(state: FileStoreState, name: string): Promise<FileStoreState>
  restoreFile(state: FileStoreState, name: string): Promise<FileStoreState>
  updateActiveContent(state: FileStoreState, content: string): FileStoreState
  saveContent(state: FileStoreState): Promise<void>
  status(): SaveStatus
}
