import { useEffect, useState } from 'react'
import type { DisplaySaveStatus } from '../hooks/useStorageBackend'

const STATUS_LABEL: Record<Exclude<DisplaySaveStatus, 'unsaved'>, string> = {
  waiting: '',
  attention: '',
  idle: '',
  saving: 'Saving…',
  saved: 'Saved',
  error: 'Save failed',
  offline: 'Offline',
}

/** Ticks once a second while `deadline` is non-null, so a rendered countdown
 * stays in sync without the parent re-rendering on every store change. */
function useCountdownSeconds(deadline: number | null): number | null {
  const [now, setNow] = useState(() => Date.now())

  useEffect(() => {
    if (deadline === null) return
    const interval = setInterval(() => setNow(Date.now()), 1_000)
    return () => clearInterval(interval)
  }, [deadline])

  return deadline === null
    ? null
    : Math.max(0, Math.ceil((deadline - now) / 1_000))
}

export function SaveStatusBadge({
  status,
  autosaveDeadline,
  label: outboxLabel = null,
}: {
  status: DisplaySaveStatus
  /** Outbox-derived text; wins over the per-status default. */
  label?: string | null
  autosaveDeadline: number | null
}) {
  const remainingSeconds = useCountdownSeconds(
    status === 'unsaved' ? autosaveDeadline : null,
  )
  const label =
    status === 'unsaved'
      ? remainingSeconds !== null
        ? `Unsaved (autosaving in ${remainingSeconds}s)`
        : 'Unsaved'
      : (outboxLabel ?? STATUS_LABEL[status])
  if (!label) return null
  return (
    <span
      className={`file-tab-bar-save-status file-tab-bar-save-status--${status}`}
      data-testid="save-status-badge"
    >
      {label}
    </span>
  )
}
