import { ChevronDownIcon } from '@radix-ui/react-icons'
import type { ReactNode } from 'react'
import { useRef } from 'react'
import { useDismissableOpen } from '../hooks/useDismissableOpen'

export interface ExportMenuItem {
  key: string
  label: string
  busyLabel: string
  busy: boolean
  disabled: boolean
  onSelect: () => void
  icon?: ReactNode
}

interface ExportMenuButtonProps {
  label: string
  icon?: ReactNode
  items: ExportMenuItem[]
  disabled?: boolean
}

export function ExportMenuButton({
  label,
  icon,
  items,
  disabled = false,
}: ExportMenuButtonProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const [open, setOpen] = useDismissableOpen(containerRef)

  return (
    <div className="export-menu" ref={containerRef}>
      <button
        type="button"
        className="preview-export-btn"
        aria-haspopup="menu"
        aria-expanded={open}
        disabled={disabled}
        onClick={() => setOpen((prev) => !prev)}
      >
        {icon}
        {label}
        <ChevronDownIcon className="export-menu-caret" aria-hidden="true" />
      </button>
      {open && !disabled ? (
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
              {item.icon}
              {item.busy ? item.busyLabel : item.label}
            </button>
          ))}
        </div>
      ) : null}
    </div>
  )
}
