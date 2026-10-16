import type { SyncedShareViewerStatus } from '../hooks/useSyncedShareViewer'

interface SyncedShareBannerProps {
  status: SyncedShareViewerStatus
  filename: string | null
  /** Name of the shared folder when the link is a folder link. */
  folderName: string | null
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
  folderName,
  ownerLogin,
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
            {ownerLogin && <span> — Shared by @{ownerLogin}</span>}
          </>
        )}
        {status === 'synced' && !folderName && (
          <>
            Synced: <strong>{filename}</strong>
            {ownerLogin && <span> — Shared by @{ownerLogin}</span>}
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
