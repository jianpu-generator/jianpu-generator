import { useCallback, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import { selectionAll, selectionToggle } from '../fileStoreQueries'

export interface FileSelection {
  /** Select mode: the file list shows checkboxes and the selection bar. */
  selecting: boolean
  /** The ticked file paths. Rust decides what a folder's checkbox means. */
  selected: string[]
  start: () => void
  /** Leaves select mode and forgets the selection. */
  stop: () => void
  /** Keeps select mode but unticks everything. */
  clear: () => void
  /** Clicking the checkbox of a file or folder row. */
  toggle: (key: string) => void
  selectAll: () => void
}

/** The file browser's multi-select: only the raw ticked paths live here;
 * toggling a folder, the units to act on and the move rules are Rust's. */
export function useFileSelection(store: FileStoreState): FileSelection {
  const [selecting, setSelecting] = useState(false)
  const [selected, setSelected] = useState<string[]>([])
  return {
    selecting,
    selected,
    start: useCallback(() => setSelecting(true), []),
    stop: useCallback(() => {
      setSelecting(false)
      setSelected([])
    }, []),
    clear: useCallback(() => setSelected([]), []),
    toggle: (key) =>
      setSelected((previous) => selectionToggle(store, previous, key)),
    selectAll: () => setSelected(selectionAll(store)),
  }
}
