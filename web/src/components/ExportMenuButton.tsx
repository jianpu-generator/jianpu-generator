import { useRef } from 'react'
import { useDismissableOpen } from '../hooks/useDismissableOpen'

export interface ExportMenuItem {
  key: string
  label: string
  busyLabel: string
  busy: boolean
  disabled: boolean
  onSelect: () => void
}

interface ExportMenuButtonProps {
  label: string
  items: ExportMenuItem[]
}

export function ExportMenuButton({ label, items }: ExportMenuButtonProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const [open, setOpen] = useDismissableOpen(containerRef)

  return (
    <div className="export-menu" ref={containerRef}>
      <button
        type="button"
        className="preview-export-btn"
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen((prev) => !prev)}
      >
        {label}
        <span className="export-menu-caret" aria-hidden="true">
          ▾
        </span>
      </button>
      {open ? (
        <div className="export-menu-list" role="menu">
          {items.map((item) => (
            <button
              key={item.key}
              type="button"
              role="menuitem"
              className="export-menu-item"
              disabled={item.disabled}
              onClick={() => {
                setOpen(false)
                item.onSelect()
              }}
            >
              {item.busy ? item.busyLabel : item.label}
            </button>
          ))}
        </div>
      ) : null}
    </div>
  )
}
