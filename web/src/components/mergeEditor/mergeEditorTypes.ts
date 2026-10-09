export interface MergeEditorProps {
  fileName: string
  /** The server's version, shown read-only on the left. */
  theirs: string
  /** Starting text of the editable result on the right. */
  initialResult: string
  /** Whether `initialResult` contains conflict markers (initial Save state). */
  hasConflictMarkers: boolean
  onSave: (result: string) => void
  onKeepBoth: () => void
  onDiscard: () => void
}

const MARKER_LINE = /^(<<<<<<<|=======|>>>>>>>)/m

/** True while the text still contains a conflict marker line. */
export function containsConflictMarkers(text: string): boolean {
  return MARKER_LINE.test(text)
}
