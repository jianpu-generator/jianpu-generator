import type { SyncedShareViewerStatus } from '../hooks/useSyncedShareViewer'

interface SyncedShareBannerProps {
  status: SyncedShareViewerStatus
  filename: string | null
  onImport: () => void
}

export function SyncedShareBanner({
  status,
  filename,
  onImport,
}: SyncedShareBannerProps) {
  return (
    <div className="shared-preview-banner">
      <p>
        {status === 'loading' && 'Loading synced file…'}
        {status === 'synced' && (
          <>
            Synced: <strong>{filename}</strong>
          </>
        )}
        {status === 'unreachable' &&
          'Could not load this synced file — try reloading.'}
        {status === 'ended' && 'This synced share has ended.'}
      </p>
      {filename && (
        <div className="shared-preview-actions">
          <button
            type="button"
            className="shared-preview-import-btn"
            onClick={onImport}
          >
            Import to my scores
          </button>
        </div>
      )}
    </div>
  )
}
