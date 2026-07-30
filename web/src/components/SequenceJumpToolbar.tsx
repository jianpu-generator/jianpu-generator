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
            touchAction: dragStartIndex !== null ? 'none' : undefined,
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
          onTouchMove={(e) => {
            if (dragStartIndex === null) return
            const touch = e.touches[0]
            const target = document.elementFromPoint(
              touch.clientX,
              touch.clientY,
            )
            const indexAttr = target
              ?.closest('[data-sequence-index]')
              ?.getAttribute('data-sequence-index')
            if (indexAttr === null || indexAttr === undefined) return
            const index = Number(indexAttr)
            setDragCurrentIndex(index)
            handleSequenceEntryRangeSelect(dragStartIndex, index)
          }}
          onTouchEnd={() => {
            setDragStartIndex(null)
            setDragCurrentIndex(null)
          }}
          onTouchCancel={() => {
            setDragStartIndex(null)
            setDragCurrentIndex(null)
          }}
        >
          {sequenceEntries.map((entry, index) => (
            <button
              // biome-ignore lint/suspicious/noArrayIndexKey: two entries (an omission and its later repeat) can share the same start_measure_index, so the array index is the only stable key
              key={index}
              type="button"
              data-sequence-index={index}
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
              onTouchStart={() => {
                setDragStartIndex(index)
                setDragCurrentIndex(index)
                handleSequenceEntryClick(index)
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
