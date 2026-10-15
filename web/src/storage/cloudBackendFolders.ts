import {
  type FileStoreState,
  deleteFolder as pureDeleteFolder,
  moveFolder as pureMoveFolder,
} from '../fileStore'
import type { OutboxLooper } from './outbox/outboxLooper'

interface FolderOperationDependencies {
  ensureWasm: () => Promise<void>
  whenReady: () => Promise<void>
  looper: OutboxLooper
  activeFileId: (state: FileStoreState, name: string) => string
  activeIdByName: Map<string, string>
  trashedIdByName: Map<string, string>
}

/** Folder rename/move and delete for the cloud backend. Each file inside is
 * renamed or trashed on its own (one outbox message per file, not one
 * batch), and explicit empty folders change only in local state -- real
 * folder semantics are Phase 3. */
export function createFolderOperations({
  ensureWasm,
  whenReady,
  looper,
  activeFileId,
  activeIdByName,
  trashedIdByName,
}: FolderOperationDependencies) {
  return {
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
        if (change.kind !== 'file') continue
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
      return nextState
    },
  }
}
