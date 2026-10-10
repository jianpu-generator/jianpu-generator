import type { ReactElement } from 'react'
import './SyncPanel.css'
import type {
  LaneView,
  LaneViewStatus,
  SyncPanelProps,
} from './syncPanelTypes.ts'

const STATUS_LABEL: Record<LaneViewStatus, string> = {
  syncing: 'Syncing',
  waiting: 'Waiting to sync',
  'possibly-stuck': 'Possibly stuck',
  'needs-sign-in': 'Needs sign-in',
  'needs-merge': 'Needs merge',
  failed: "Couldn't sync",
}

const RETRYABLE: ReadonlySet<LaneViewStatus> = new Set([
  'waiting',
  'possibly-stuck',
  'failed',
])

function renderLane(lane: LaneView, props: SyncPanelProps): ReactElement {
  const { fileId } = lane
  return (
    <li
      key={fileId}
      className="sync-panel__lane"
      data-testid={`sync-lane-${fileId}`}
    >
      <div className="sync-panel__header">
        <span className="sync-panel__file-name">{lane.fileName}</span>
        <span
          className={`sync-panel__pill sync-panel__pill--${lane.status}`}
          data-status={lane.status}
        >
          {STATUS_LABEL[lane.status]}
        </span>
      </div>
      <p className="sync-panel__sentence">{lane.sentence}</p>
      <div className="sync-panel__actions">
        {RETRYABLE.has(lane.status) && (
          <button type="button" onClick={() => props.onRetryNow(fileId)}>
            Retry now
          </button>
        )}
        {lane.status === 'needs-merge' && (
          <>
            <button
              type="button"
              onClick={() => props.onReviewAndMerge(fileId)}
            >
              Review and merge
            </button>
            <button type="button" onClick={() => props.onKeepBoth(fileId)}>
              Keep both files
            </button>
          </>
        )}
        {(lane.status === 'needs-merge' || lane.status === 'failed') && (
          <button type="button" onClick={() => props.onDiscard(fileId)}>
            Discard
          </button>
        )}
        {(lane.status === 'needs-merge' || lane.status === 'failed') && (
          <button type="button" onClick={() => props.onDownloadCopy(fileId)}>
            Download my copy
          </button>
        )}
      </div>
      <details>
        <summary>Details</summary>
        <h4>Queued messages</h4>
        <ul className="sync-panel__details-list">
          {lane.messages.map((message, index) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: messages have no stable id in the view
            <li key={index}>
              {message.kindLabel}, {message.sizeLabel}, {message.createdLabel}
            </li>
          ))}
        </ul>
        <h4>Attempts</h4>
        <ul className="sync-panel__details-list">
          {lane.attempts.map((attempt, index) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: attempts have no stable id in the view
            <li key={index}>
              {attempt.atLabel}: {attempt.outcomeLabel}
            </li>
          ))}
        </ul>
      </details>
    </li>
  )
}

export function SyncPanel(props: SyncPanelProps): ReactElement {
  return (
    <div>
      <ul className="sync-panel">
        {props.lanes.map((lane) => renderLane(lane, props))}
      </ul>
      <button type="button" onClick={() => props.onCopyDiagnostics()}>
        Copy diagnostics
      </button>
    </div>
  )
}
