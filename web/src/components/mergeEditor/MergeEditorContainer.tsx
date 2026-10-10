import * as Dialog from '@radix-ui/react-dialog'
import type { ReactElement } from 'react'
import type { ConflictDetails } from '../../storage/outbox/conflictFlow'
import type { OutboxLooper } from '../../storage/outbox/outboxLooper'
import { MergeEditor } from './MergeEditor.tsx'
import { containsConflictMarkers } from './mergeEditorTypes.ts'

export interface MergeEditorContainerProps {
  /** The conflict being settled; the dialog is closed while this is null. */
  conflict: ConflictDetails | null
  fileName: string
  looper: OutboxLooper
  onKeepBoth: (fileId: string) => Promise<void>
  /** Discards the queued edits through the usual confirmation dialog. */
  onDiscard: (fileId: string) => void
  onClose: () => void
}

/** The three ways out of a merge, kept apart from React so they can be tested. */
export function mergeExits(props: MergeEditorContainerProps) {
  const { conflict, looper } = props
  return {
    async save(result: string): Promise<void> {
      if (!conflict) return
      await looper.resolve(conflict.fileId, {
        tag: 'merged-and-save',
        val: { content: result, serverRevision: conflict.serverRevision },
      })
      props.onClose()
    },
    async keepBoth(): Promise<void> {
      if (!conflict) return
      await props.onKeepBoth(conflict.fileId)
      props.onClose()
    },
    discard(): void {
      if (!conflict) return
      props.onClose()
      props.onDiscard(conflict.fileId)
    },
  }
}

export function MergeEditorContainer(
  props: MergeEditorContainerProps,
): ReactElement {
  const { conflict } = props
  const exits = mergeExits(props)
  return (
    <Dialog.Root
      open={conflict !== null}
      onOpenChange={(open) => {
        if (!open) props.onClose()
      }}
    >
      <Dialog.Portal>
        <Dialog.Overlay className="sync-panel__overlay" />
        <Dialog.Content
          className="sync-panel__dialog"
          data-testid="merge-dialog"
          aria-describedby={undefined}
        >
          <Dialog.Title>Review and merge</Dialog.Title>
          {conflict && (
            <MergeEditor
              fileName={props.fileName}
              theirs={conflict.theirs}
              initialResult={conflict.mergedWithMarkers}
              hasConflictMarkers={containsConflictMarkers(
                conflict.mergedWithMarkers,
              )}
              onSave={(result) => void exits.save(result)}
              onKeepBoth={() => void exits.keepBoth()}
              onDiscard={exits.discard}
            />
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
