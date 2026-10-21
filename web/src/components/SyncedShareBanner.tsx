import type { SyncedShareViewerStatus } from '../hooks/useSyncedShareViewer'

interface SyncedShareBannerProps {
  status: SyncedShareViewerStatus
  filename: string | null
  /** Name of the shared folder when the link is a folder link. */
  folderName: string | null
  onImport: () => void
}

export function SyncedShareBanner({
  status,
  filename,
  folderName,
  onImport,
}: SyncedShareBannerProps) {
  const canImport = status === 'synced' && (folderName || filename)
  return (
    <div className="shared-preview-banner">
      <p>
        {status === 'loading' && 'Loading synced share…'}
        {status === 'synced' && folderName && (
          <>
            Synced folder: <strong>{folderName}</strong>
          </>
        )}
        {status === 'synced' && !folderName && (
          <>
            Synced: <strong>{filename}</strong>
          </>
        )}
        {status === 'unreachable' &&
          'Could not load this synced share — try reloading.'}
        {status === 'too-large' && 'This shared folder is too large to open.'}
        {status === 'ended' && 'This synced share has ended.'}
      </p>
      {canImport && (
        <div className="shared-preview-actions">
          <button
            type="button"
            className="shared-preview-import-btn"
            onClick={onImport}
          >
            {folderName ? 'Import this folder' : 'Import to my scores'}
          </button>
        </div>
      )}
    </div>
  )
}
