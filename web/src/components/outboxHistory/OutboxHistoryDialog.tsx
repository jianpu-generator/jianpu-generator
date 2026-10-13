import * as Dialog from '@radix-ui/react-dialog'
import { type ReactElement, useEffect, useMemo, useState } from 'react'
import type { FileName } from '../../jianpuWasm'
import type { OutboxLooper } from '../../storage/outbox/outboxLooper'
import {
  EMPTY_FILTER_STATE,
  type HistoryFilterState,
  toHistoryFilter,
} from './historyFilterState'
import { OutboxHistoryView } from './OutboxHistoryView'

export interface OutboxHistoryDialogProps {
  open: boolean
  onOpenChange(open: boolean): void
  looper: OutboxLooper
  files: FileName[]
}

export function OutboxHistoryDialog(
  props: OutboxHistoryDialogProps,
): ReactElement {
  const { looper, open, files } = props
  const [filter, setFilter] = useState<HistoryFilterState>(EMPTY_FILTER_STATE)
  // Bumped whenever the outbox publishes a new state, to re-read the rows.
  const [revision, setRevision] = useState(0)
  useEffect(() => {
    if (!open) return
    void looper.refreshHistory()
    return looper.subscribe(() => setRevision((current) => current + 1))
  }, [looper, open])
  // biome-ignore lint/correctness/useExhaustiveDependencies: `revision` re-reads the looper's rows
  const rows = useMemo(
    () => looper.historyRows(files, toHistoryFilter(filter)),
    [looper, files, filter, revision],
  )
  // biome-ignore lint/correctness/useExhaustiveDependencies: `revision` re-reads the looper's rows
  const totalCount = useMemo(
    () => looper.historyRows(files, toHistoryFilter(EMPTY_FILTER_STATE)).length,
    [looper, files, revision],
  )
  return (
    <Dialog.Root open={open} onOpenChange={props.onOpenChange}>
      <Dialog.Portal>
        <Dialog.Content
          className="outbox-history__dialog"
          data-testid="outbox-history"
          aria-describedby={undefined}
        >
          <header className="outbox-history__header">
            <Dialog.Title>Outbox history</Dialog.Title>
            <Dialog.Close asChild>
              <button type="button">Close</button>
            </Dialog.Close>
          </header>
          <OutboxHistoryView
            rows={rows}
            totalCount={totalCount}
            files={files}
            filter={filter}
            onFilterChange={setFilter}
          />
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
