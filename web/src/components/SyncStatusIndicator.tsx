import type { SyncStatus } from '../hooks/useGitHubAutosave'
import './SyncStatusIndicator.css'

export interface SyncStatusIndicatorProps {
  status: SyncStatus
  error: string | null
}

function statusLabel(status: SyncStatus, error: string | null): string | null {
  switch (status) {
    case 'saving':
      return 'Saving to GitHub…'
    case 'saved':
      return 'Saved to GitHub'
    case 'error':
      return error ?? 'Failed to sync with GitHub'
    default:
      return null
  }
}

export function SyncStatusIndicator({
  status,
  error,
}: SyncStatusIndicatorProps) {
  const label = statusLabel(status, error)
  if (!label) {
    return null
  }

  return (
    <div
      className={[
        'sync-status-indicator',
        status === 'error' ? 'sync-status-indicator--error' : '',
        status === 'saved' ? 'sync-status-indicator--saved' : '',
      ]
        .filter(Boolean)
        .join(' ')}
      role={status === 'error' ? 'alert' : 'status'}
      data-testid="sync-status"
      data-sync-status={status}
    >
      {label}
    </div>
  )
}
