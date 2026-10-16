import { useCallback, useState } from 'react'
import { type FileStoreState, mergeBackendResult } from '../fileStore'
import type { StorageBackend } from '../storage/types'

export interface FileOpError {
  title: string
  message: string
  stack?: string
}

export function useFileOperations(
  store: FileStoreState,
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void,
  backend: StorageBackend,
) {
  const [creatingFile, setCreatingFile] = useState(false)
  const [deletingFileName, setDeletingFileName] = useState<string | null>(null)
  const [creatingFolder, setCreatingFolder] = useState(false)
  const [duplicatingFile, setDuplicatingFile] = useState(false)
  const [renamingFileName, setRenamingFileName] = useState<string | null>(null)
  const [restoringFileName, setRestoringFileName] = useState<string | null>(
    null,
  )
  const [fileOpError, setFileOpError] = useState<FileOpError | null>(null)

  const runFileOp = useCallback(
    async (
      errorTitle: string,
      setPending: (pending: boolean) => void,
      op: (base: FileStoreState) => Promise<FileStoreState>,
    ): Promise<FileStoreState | undefined> => {
      const base = store
      setPending(true)
      try {
        const next = await op(base)
        setStore((prev) => mergeBackendResult(prev, base, next))
        return next
      } catch (error) {
        setFileOpError({
          title: errorTitle,
          message: error instanceof Error ? error.message : String(error),
          stack: error instanceof Error ? error.stack : undefined,
        })
      } finally {
        setPending(false)
      }
      return undefined
    },
    [setStore, store],
  )

  const handleCreate = useCallback(
    (folder: string | undefined, name: string) =>
      runFileOp('Could not create file', setCreatingFile, (base) =>
        backend.createFile(base, folder, name),
      ),
    [runFileOp, backend],
  )

  /** Resolves to the new store, so the caller can reveal the new folder. */
  const handleCreateFolder = useCallback(
    (parent: string | undefined, name: string) =>
      runFileOp('Could not create folder', setCreatingFolder, (base) =>
        backend.createFolder(base, parent, name),
      ),
    [runFileOp, backend],
  )

  const handleDuplicate = useCallback(
    () =>
      runFileOp('Could not duplicate file', setDuplicatingFile, (base) =>
        backend.duplicateFile(base),
      ),
    [runFileOp, backend],
  )

  const handleRename = useCallback(
    (from: string, to: string) =>
      runFileOp(
        'Could not rename file',
        (pending) => setRenamingFileName(pending ? from : null),
        (base) => backend.renameFile(base, from, to),
      ),
    [runFileOp, backend],
  )

  const handleDelete = useCallback(
    (name: string) =>
      runFileOp(
        'Could not delete file',
        (pending) => setDeletingFileName(pending ? name : null),
        (base) => backend.deleteFile(base, name),
      ),
    [runFileOp, backend],
  )

  const handleMoveFolder = useCallback(
    (from: string, to: string) =>
      runFileOp(
        'Could not move folder',
        (pending) => setRenamingFileName(pending ? from : null),
        (base) => backend.moveFolder(base, from, to),
      ),
    [runFileOp, backend],
  )

  const handleDeleteFolder = useCallback(
    (folder: string) =>
      runFileOp(
        'Could not delete folder',
        (pending) => setDeletingFileName(pending ? folder : null),
        (base) => backend.deleteFolder(base, folder),
      ),
    [runFileOp, backend],
  )

  const handleRestore = useCallback(
    (name: string) =>
      runFileOp(
        'Could not restore file',
        (pending) => setRestoringFileName(pending ? name : null),
        (base) => backend.restoreFile(base, name),
      ),
    [runFileOp, backend],
  )

  return {
    creatingFile,
    creatingFolder,
    deletingFileName,
    duplicatingFile,
    renamingFileName,
    restoringFileName,
    fileOpError,
    setFileOpError,
    handleCreate,
    handleCreateFolder,
    handleDuplicate,
    handleRename,
    handleDelete,
    handleMoveFolder,
    handleDeleteFolder,
    handleRestore,
  }
}
