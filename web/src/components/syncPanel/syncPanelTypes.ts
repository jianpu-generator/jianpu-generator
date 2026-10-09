export type LaneViewStatus =
  | 'syncing'
  | 'waiting'
  | 'possibly-stuck'
  | 'needs-sign-in'
  | 'needs-merge'
  | 'failed'

export interface AttemptView {
  atLabel: string
  outcomeLabel: string
}

export interface MessageView {
  kindLabel: string
  sizeLabel: string
  createdLabel: string
}

export interface LaneView {
  fileId: string
  fileName: string
  status: LaneViewStatus
  /** Plain-language line, from the copy table. */
  sentence: string
  retryInSeconds?: number
  messages: MessageView[]
  attempts: AttemptView[]
}

export interface SyncPanelProps {
  lanes: LaneView[]
  onRetryNow(fileId: string): void
  onReviewAndMerge(fileId: string): void
  onKeepBoth(fileId: string): void
  onDiscard(fileId: string): void
  onDownloadCopy(fileId: string): void
  onCopyDiagnostics(): void
}
