import { useState } from 'react'
import type { FileStoreState } from '../fileStore'
import { createFolderError, foldersToReveal } from '../fileStoreQueries'

interface NewFolderPromptInput {
  store: FileStoreState
  onCreateFolder: (
    parent: string | undefined,
    name: string,
  ) => Promise<FileStoreState | undefined>
  expandFolders: (folders: string[]) => void
}

/** State and wiring for the New folder name prompt: `open(parent)` shows it,
 * `dialogProps` go to `NewFolderDialog`. A confirmed name creates the folder
 * and opens it, with the folders above it. */
export function useNewFolderPrompt({
  store,
  onCreateFolder,
  expandFolders,
}: NewFolderPromptInput) {
  const [target, setTarget] = useState<{ parent?: string } | null>(null)
  return {
    open: (parent?: string) => setTarget({ parent }),
    dialogProps: {
      open: target !== null,
      onOpenChange: (open: boolean) => {
        if (!open) setTarget(null)
      },
      parent: target?.parent,
      createError: (parent: string | undefined, name: string) =>
        createFolderError(store, parent, name),
      onCreate: async (parent: string | undefined, name: string) => {
        const next = await onCreateFolder(parent, name)
        const known = new Set(store.folders)
        expandFolders(
          (next?.folders ?? [])
            .filter((created) => !known.has(created))
            .flatMap(foldersToReveal),
        )
      },
    },
  }
}
