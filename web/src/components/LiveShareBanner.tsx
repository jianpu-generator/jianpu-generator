import type { LiveViewerStatus } from '../hooks/useLiveViewer'

interface LiveShareBannerProps {
  status: LiveViewerStatus
  filename: string | null
  onImport: () => void
}

export function LiveShareBanner({
  status,
  filename,
  onImport,
}: LiveShareBannerProps) {
  return (
    <div className="shared-preview-banner">
      <p>
        {status === 'connecting' && 'Connecting to live session…'}
        {status === 'live' && (
          <>
            Live: <strong>{filename}</strong>
          </>
        )}
        {status === 'disconnected' && 'Connection lost — reconnecting…'}
        {status === 'ended' && 'This live session has ended.'}
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
