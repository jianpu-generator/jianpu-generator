import type { SyncedShareViewerStatus } from '../hooks/useSyncedShareViewer'

interface SyncedShareBannerProps {
  status: SyncedShareViewerStatus
  filename: string | null
  /** The owning user's cached GitHub login (`user_identities.login`, via
   * the worker's public `GET /shares/:shareId` response's `ownerLogin`) --
   * `null` when none is cached for this share's owner. Shown as "Shared by
   * @username" per task 10; viewing stays fully anonymous regardless (no
   * login required to see this). */
  ownerLogin: string | null
  onImport: () => void
}

export function SyncedShareBanner({
  status,
  filename,
  ownerLogin,
  onImport,
}: SyncedShareBannerProps) {
  return (
    <div className="shared-preview-banner">
      <p>
        {status === 'loading' && 'Loading synced file…'}
        {status === 'synced' && (
          <>
            Synced: <strong>{filename}</strong>
            {ownerLogin && <span> — Shared by @{ownerLogin}</span>}
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
