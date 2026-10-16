import {
  type FileStoreState,
  createFolder as pureCreateFolder,
  deleteFolder as pureDeleteFolder,
  moveFolder as pureMoveFolder,
} from '../fileStore'
import type { CloudPlaceholders } from './cloudBackendPlaceholders'
import type { OutboxLooper } from './outbox/outboxLooper'

interface FolderOperationDependencies {
  ensureWasm: () => Promise<void>
  whenReady: () => Promise<void>
  looper: OutboxLooper
  placeholders: CloudPlaceholders
  activeFileId: (state: FileStoreState, name: string) => string
  activeIdByName: Map<string, string>
  trashedIdByName: Map<string, string>
}

/** Folder create, rename/move and delete for the cloud backend. Each file
 * inside is renamed or trashed on its own (one outbox message per file, not
 * one batch). Explicit folders are persisted as placeholder files
 * (`cloudBackendPlaceholders.ts`): created with the folder, renamed with it
 * and trashed with it; a placeholder left behind in a folder that later got
 * real files is harmless and stays. */
export function createFolderOperations({
  ensureWasm,
  whenReady,
  looper,
  placeholders,
  activeFileId,
  activeIdByName,
  trashedIdByName,
}: FolderOperationDependencies) {
  return {
    async createFolder(
      state: FileStoreState,
      parent?: string,
    ): Promise<FileStoreState> {
      await ensureWasm()
      const nextState = pureCreateFolder(state, parent)
      const before = state.folders ?? []
      for (const folder of nextState.folders ?? []) {
        if (!before.includes(folder)) await placeholders.create(folder)
      }
      return nextState
    },

    async moveFolder(
      state: FileStoreState,
      from: string,
      to: string,
    ): Promise<FileStoreState> {
      await ensureWasm()
      const result = pureMoveFolder(state, from, to)
      if (!result.ok) return state
      await whenReady()
      for (const change of result.changes) {
        if (change.kind === 'explicit-folder') {
          await placeholders.rename(change.oldPath, change.newPath)
          continue
        }
        const id = activeFileId(state, change.oldPath)
        activeIdByName.delete(change.oldPath)
        activeIdByName.set(change.newPath, id)
        await looper.enqueue(id, {
          tag: 'rename-file',
          val: { to: change.newPath },
        })
      }
      return result.state
    },

    async deleteFolder(
      state: FileStoreState,
      folder: string,
    ): Promise<FileStoreState> {
      await ensureWasm()
      const nextState = pureDeleteFolder(state, folder)
      if (nextState === state) return nextState
      await whenReady()
      for (const name of Object.keys(state.userFiles)) {
        if (name in nextState.userFiles) continue
        const id = activeFileId(state, name)
        activeIdByName.delete(name)
        trashedIdByName.set(name, id)
        await looper.enqueue(id, { tag: 'trash-file' })
      }
      const remaining = nextState.folders ?? []
      for (const folder of state.folders ?? []) {
        if (!remaining.includes(folder)) await placeholders.trash(folder)
      }
      return nextState
    },
  }
}
