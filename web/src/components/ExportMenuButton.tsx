import { useEffect, useRef, useState } from 'react'

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
  const [open, setOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!open) return
    const handleClickOutside = (event: MouseEvent) => {
      if (!containerRef.current?.contains(event.target as Node)) {
        setOpen(false)
      }
    }
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setOpen(false)
    }
    document.addEventListener('mousedown', handleClickOutside)
    document.addEventListener('keydown', handleKeyDown)
    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [open])

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
