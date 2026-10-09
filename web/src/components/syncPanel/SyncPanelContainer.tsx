import * as Dialog from '@radix-ui/react-dialog'
import { useEffect, useState } from 'react'
import type { FileStoreState } from '../../fileStore'
import type { CloudBackend } from '../../storage/cloudBackendTypes'
import type { OutboxSnapshot } from '../../storage/outbox/outboxLooper'
import type { StorageBackend } from '../../storage/types'
import { buildDiagnostics, copyDiagnostics } from './copyDiagnostics'
import { DiscardConfirmDialog } from './DiscardConfirmDialog'
import { downloadLocalCopy } from './downloadLocalCopy'
import { toLaneViews } from './laneViewMapping'
import { SyncPanel } from './SyncPanel'

export interface SyncPanelContainerProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  backend: StorageBackend
  store: FileStoreState
  /** Puts the server's text into the open store, replacing the local one. */
  onServerContent: (fileId: string, content: string) => void
  onReviewAndMerge?: (fileId: string) => void
  onKeepBoth?: (fileId: string) => void
}

function nameOfFile(store: FileStoreState, fileId: string): string {
  return (
    Object.entries(store.fileIds).find(([, id]) => id === fileId)?.[0] ?? fileId
  )
}

function useOutboxSnapshot(backend: CloudBackend): OutboxSnapshot {
  const looper = backend.outbox()
  const [snapshot, setSnapshot] = useState(() => looper.snapshot())
  useEffect(() => {
    setSnapshot(looper.snapshot())
    return looper.subscribe(setSnapshot)
  }, [looper])
  return snapshot
}

function CloudSyncPanel(
  props: SyncPanelContainerProps & { backend: CloudBackend },
) {
  const { backend, store } = props
  const snapshot = useOutboxSnapshot(backend)
  const [discardingFileId, setDiscardingFileId] = useState<string | null>(null)
  const lanes = toLaneViews(snapshot, (fileId) => nameOfFile(store, fileId))

  const download = (fileId: string) => {
    const name = nameOfFile(store, fileId)
    downloadLocalCopy(name, store.userFiles[name] ?? store.bin[name] ?? '')
  }

  const discard = async (fileId: string) => {
    setDiscardingFileId(null)
    await backend.outbox().resolve(fileId, { tag: 'discard' })
    const server = await backend.fetchServerFile(fileId)
    if (server) props.onServerContent(fileId, server.content)
  }

  return (
    <>
      <Dialog.Root open={props.open} onOpenChange={props.onOpenChange}>
        <Dialog.Portal>
          <Dialog.Overlay className="sync-panel__overlay" />
          <Dialog.Content
            className="sync-panel__dialog"
            data-testid="sync-panel"
            aria-describedby={undefined}
          >
            <Dialog.Title>Sync status</Dialog.Title>
            <SyncPanel
              lanes={lanes}
              onRetryNow={(fileId) =>
                void backend.outbox().resolve(fileId, { tag: 'retry' })
              }
              onReviewAndMerge={(fileId) => props.onReviewAndMerge?.(fileId)}
              onKeepBoth={(fileId) => props.onKeepBoth?.(fileId)}
              onDiscard={setDiscardingFileId}
              onDownloadCopy={download}
              onCopyDiagnostics={() =>
                void copyDiagnostics(buildDiagnostics(lanes))
              }
            />
            <Dialog.Close asChild>
              <button type="button">Close</button>
            </Dialog.Close>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
      <DiscardConfirmDialog
        open={discardingFileId !== null}
        fileName={nameOfFile(store, discardingFileId ?? '')}
        onDownloadFirst={() => {
          if (discardingFileId) download(discardingFileId)
        }}
        onConfirm={() => {
          if (discardingFileId) void discard(discardingFileId)
        }}
        onCancel={() => setDiscardingFileId(null)}
      />
    </>
  )
}

export function SyncPanelContainer(props: SyncPanelContainerProps) {
  const { backend } = props
  if (backend.kind !== 'cloud') return null
  return <CloudSyncPanel {...props} backend={backend as CloudBackend} />
}
