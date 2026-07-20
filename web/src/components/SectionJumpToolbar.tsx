interface SectionJumpToolbarProps {
  sectionLabels: string[]
  dragStartLabel: string | null
  setDragStartLabel: (label: string | null) => void
  setDragCurrentLabel: (label: string | null) => void
  activeHighlightedLabels: Set<string>
  handleSectionJump: (label: string) => void
  handleSectionRangeSelect: (labelA: string, labelB: string) => void
}

export function SectionJumpToolbar({
  sectionLabels,
  dragStartLabel,
  setDragStartLabel,
  setDragCurrentLabel,
  activeHighlightedLabels,
  handleSectionJump,
  handleSectionRangeSelect,
}: SectionJumpToolbarProps) {
  if (sectionLabels.length === 0) return null

  return (
    <div className="workspace-toolbar">
      <span className="workspace-toolbar-label">Sections</span>
      <div
        role="toolbar"
        className="workspace-toolbar-sections"
        style={{
          userSelect: dragStartLabel !== null ? 'none' : undefined,
        }}
        onMouseDown={(e) => e.preventDefault()}
        onMouseUp={() => {
          setDragStartLabel(null)
          setDragCurrentLabel(null)
        }}
        onMouseLeave={() => {
          setDragStartLabel(null)
          setDragCurrentLabel(null)
        }}
      >
        {sectionLabels.map((label) => (
          <button
            key={label}
            type="button"
            className={[
              'section-jump-btn',
              activeHighlightedLabels.has(label)
                ? 'section-jump-btn--dragging'
                : '',
            ].join(' ')}
            style={{
              cursor: dragStartLabel !== null ? 'ew-resize' : undefined,
            }}
            onMouseDown={() => {
              setDragStartLabel(label)
              setDragCurrentLabel(label)
              handleSectionJump(label)
            }}
            onMouseEnter={() => {
              if (dragStartLabel !== null) {
                setDragCurrentLabel(label)
                handleSectionRangeSelect(dragStartLabel, label)
              }
            }}
          >
            {label}
          </button>
        ))}
      </div>
    </div>
  )
}
