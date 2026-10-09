import type { OutboxSnapshot } from '../storage/outbox/outboxLooper'

export type OutboxBadgeStatus = 'saved' | 'saving' | 'waiting' | 'attention'

export interface OutboxBadge {
  status: OutboxBadgeStatus
  label: string
}

function plural(count: number, singular: string, pluralForm: string): string {
  return count === 1 ? singular : pluralForm
}

/** A halted queue (undecodable stored data) counts as one file needing
 * attention, since nothing in it can be sent. */
function attentionCount(snapshot: OutboxSnapshot): number {
  return snapshot.halted
    ? Math.max(1, snapshot.summary.lanesNeedingAttention)
    : snapshot.summary.lanesNeedingAttention
}

/** Whether leaving the page now could lose edits. */
export function hasUnsyncedWork(snapshot: OutboxSnapshot): boolean {
  return snapshot.halted !== null || snapshot.summary.hasUnsynced
}

/** Display text only; every classification comes from the Rust summary. */
export function outboxSaveBadge(snapshot: OutboxSnapshot): OutboxBadge {
  const attention = attentionCount(snapshot)
  if (attention > 0) {
    return {
      status: 'attention',
      label: `${attention} ${plural(attention, 'file needs', 'files need')} attention`,
    }
  }
  const { summary } = snapshot
  if (summary.lanes.some((lane) => lane.reason === 'waiting-to-retry')) {
    return {
      status: 'waiting',
      label: `${summary.totalPending} ${plural(summary.totalPending, 'change', 'changes')} waiting`,
    }
  }
  return summary.hasUnsynced
    ? { status: 'saving', label: 'Saving…' }
    : { status: 'saved', label: 'Saved' }
}

/**
 * Decides whether the `beforeunload` handler should warn about unsaved
 * changes. Pure for the same testability reason as `shouldScheduleAutosave`.
 * True on the cloud backend while a debounced save is armed (`isPending`) or
 * while the outbox still holds unsynced work (`hasUnsynced`, which includes
 * halted and needs-attention lanes). Never true for `localBackend`.
 */
export function shouldWarnBeforeUnload(
  backendKind: 'local' | 'cloud',
  isPending: boolean,
  hasUnsynced: boolean,
): boolean {
  if (backendKind !== 'cloud') return false
  return isPending || hasUnsynced
}
