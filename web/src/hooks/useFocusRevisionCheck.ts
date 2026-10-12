import { useEffect, useRef } from 'react'
import type { CloudBackend, ServerFile } from '../storage/cloudBackendTypes'

export interface FocusRevisionDependencies {
  fetchServerFile(fileId: string): Promise<ServerFile | undefined>
  recordedRevision(fileId: string): bigint | undefined
  hasPendingMessages(fileId: string): boolean
  /** Typed text that has not reached the outbox yet (an armed autosave). */
  hasUnsavedEdits(): boolean
  applyServerFile(
    fileId: string,
    revision: number,
    content: string,
  ): Promise<void>
}

/** Loads the server's copy of `fileId` when it is ahead of the recorded
 * revision and nothing is waiting to be sent for that file. */
export async function checkFocusRevision(
  dependencies: FocusRevisionDependencies,
  fileId: string,
): Promise<void> {
  if (dependencies.hasUnsavedEdits() || dependencies.hasPendingMessages(fileId))
    return
  const serverFile = await dependencies.fetchServerFile(fileId)
  if (!serverFile) return
  const recorded = dependencies.recordedRevision(fileId)
  if (recorded !== undefined && BigInt(serverFile.revision) <= recorded) return
  // Messages may have been queued while the fetch was in flight.
  if (dependencies.hasUnsavedEdits() || dependencies.hasPendingMessages(fileId))
    return
  await dependencies.applyServerFile(
    fileId,
    serverFile.revision,
    serverFile.content,
  )
}

function focusRevisionDependencies(
  backend: CloudBackend,
  onServerContent: (fileId: string, content: string) => void,
  hasUnsavedEdits: () => boolean,
): FocusRevisionDependencies {
  return {
    hasUnsavedEdits,
    fetchServerFile: (fileId) => backend.fetchServerFile(fileId),
    recordedRevision: (fileId) =>
      backend
        .outbox()
        .snapshot()
        .queue.revisions.find((entry) => entry.fileId === fileId)?.revision,
    hasPendingMessages: (fileId) =>
      backend
        .outbox()
        .snapshot()
        .queue.lanes.some(
          (lane) => lane.fileId === fileId && lane.messages.length > 0,
        ),
    async applyServerFile(fileId, revision, content) {
      await backend.bases().recordBase(fileId, content)
      await backend.outbox().recordRevision(fileId, BigInt(revision))
      onServerContent(fileId, content)
    },
  }
}

export function useFocusRevisionCheck(options: {
  backend: CloudBackend | undefined
  activeFileId: string | undefined
  onServerContent: (fileId: string, content: string) => void
  hasUnsavedEdits: boolean
}): void {
  const { backend, activeFileId, onServerContent } = options
  const unsavedRef = useRef(options.hasUnsavedEdits)
  unsavedRef.current = options.hasUnsavedEdits
  useEffect(() => {
    if (!backend || !activeFileId) return
    const dependencies = focusRevisionDependencies(
      backend,
      onServerContent,
      () => unsavedRef.current,
    )
    const check = () => {
      if (document.visibilityState === 'hidden') return
      checkFocusRevision(dependencies, activeFileId).catch(() => undefined)
    }
    document.addEventListener('visibilitychange', check)
    window.addEventListener('online', check)
    return () => {
      document.removeEventListener('visibilitychange', check)
      window.removeEventListener('online', check)
    }
  }, [backend, activeFileId, onServerContent])
}
