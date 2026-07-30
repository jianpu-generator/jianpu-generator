import { ChevronDown, ChevronRight } from 'lucide-react'
import { useState } from 'react'
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
  const [collapsed, setCollapsed] = useState(false)

  if (sequenceEntries.length === 0) return null

  return (
    <div
      className={[
        'workspace-toolbar',
        collapsed ? 'workspace-toolbar--collapsed' : '',
      ].join(' ')}
    >
      <button
        type="button"
        className={[
          'workspace-toolbar-label',
          'workspace-toolbar-label--toggle',
          collapsed ? 'workspace-toolbar-label--toggle-fill' : '',
        ].join(' ')}
        onClick={() => setCollapsed((value) => !value)}
        aria-expanded={!collapsed}
      >
        {collapsed ? (
          <ChevronRight size={12} aria-hidden="true" />
        ) : (
          <ChevronDown size={12} aria-hidden="true" />
        )}
        Sequence
      </button>
      {collapsed ? null : (
        <div
          role="toolbar"
          className="workspace-toolbar-sections toolbar-scroll-list"
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
      )}
    </div>
  )
}
