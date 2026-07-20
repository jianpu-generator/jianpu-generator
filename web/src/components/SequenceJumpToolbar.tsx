import type { SequenceEntry } from '../types'

interface SequenceJumpToolbarProps {
  sequenceEntries: SequenceEntry[]
  dragStartIndex: number | null
  setDragStartIndex: (index: number | null) => void
  setDragCurrentIndex: (index: number | null) => void
  activeHighlightedIndices: Set<number>
  handleSequenceEntryClick: (index: number) => void
  handleSequenceEntryRangeSelect: (indexA: number, indexB: number) => void
}

export function SequenceJumpToolbar({
  sequenceEntries,
  dragStartIndex,
  setDragStartIndex,
  setDragCurrentIndex,
  activeHighlightedIndices,
  handleSequenceEntryClick,
  handleSequenceEntryRangeSelect,
}: SequenceJumpToolbarProps) {
  if (sequenceEntries.length === 0) return null

  return (
    <div className="workspace-toolbar">
      <span className="workspace-toolbar-label">Sequence</span>
      <div
        role="toolbar"
        className="workspace-toolbar-sections"
        style={{
          userSelect: dragStartIndex !== null ? 'none' : undefined,
        }}
        onMouseDown={(e) => e.preventDefault()}
        onMouseUp={() => {
          setDragStartIndex(null)
          setDragCurrentIndex(null)
        }}
        onMouseLeave={() => {
          setDragStartIndex(null)
          setDragCurrentIndex(null)
        }}
      >
        {sequenceEntries.map((entry, index) => (
          <button
            // biome-ignore lint/suspicious/noArrayIndexKey: two entries (an omission and its later repeat) can share the same start_measure_index, so the array index is the only stable key
            key={index}
            type="button"
            className={[
              'section-jump-btn',
              activeHighlightedIndices.has(index)
                ? 'section-jump-btn--dragging'
                : '',
            ].join(' ')}
            style={{
              cursor: dragStartIndex !== null ? 'ew-resize' : undefined,
            }}
            onMouseDown={() => {
              setDragStartIndex(index)
              setDragCurrentIndex(index)
              handleSequenceEntryClick(index)
            }}
            onMouseEnter={() => {
              if (dragStartIndex !== null) {
                setDragCurrentIndex(index)
                handleSequenceEntryRangeSelect(dragStartIndex, index)
              }
            }}
          >
            {entry.label}
          </button>
        ))}
      </div>
    </div>
  )
}
