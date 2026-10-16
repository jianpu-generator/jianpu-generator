import { useState } from 'react'
import type { FileStoreState } from '../fileStore'
import { createFileError } from '../fileStoreQueries'

interface NewFilePromptInput {
  store: FileStoreState
  onCreate: (folder: string | undefined, name: string) => void
  /** Runs once a confirmed name has been handed to `onCreate`. */
  onConfirmed?: () => void
}

/** State and wiring for the New file name prompt: `open(folder)` shows it,
 * `dialogProps` go to `NewEntryDialog`. A confirmed name creates the file. */
export function useNewFilePrompt({
  store,
  onCreate,
  onConfirmed,
}: NewFilePromptInput) {
  const [target, setTarget] = useState<{ folder?: string } | null>(null)
  return {
    open: (folder?: string) => setTarget({ folder }),
    dialogProps: {
      kind: 'file' as const,
      open: target !== null,
      onOpenChange: (open: boolean) => {
        if (!open) setTarget(null)
      },
      parent: target?.folder,
      createError: (folder: string | undefined, name: string) =>
        createFileError(store, folder, name),
      onCreate: (folder: string | undefined, name: string) => {
        onCreate(folder, name)
        onConfirmed?.()
      },
    },
  }
}
