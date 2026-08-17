import { ChevronDownIcon } from '@radix-ui/react-icons'
import type { CSSProperties, ReactNode } from 'react'
import { useLayoutEffect, useRef, useState } from 'react'
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
  const buttonRef = useRef<HTMLButtonElement>(null)
  const [open, setOpen] = useDismissableOpen(containerRef)
  // The header scrolls horizontally on mobile (`overflow-x: auto`), which
  // per the CSS overflow spec forces its `overflow-y` to `auto` too,
  // clipping a plain `position: absolute` menu instead of letting it float
  // over the page. Positioning the menu as `fixed`, anchored to the
  // button's live viewport rect, escapes that clip entirely.
  const [menuStyle, setMenuStyle] = useState<CSSProperties | null>(null)

  useLayoutEffect(() => {
    if (!open) {
      setMenuStyle(null)
      return
    }
    const updatePosition = () => {
      const rect = buttonRef.current?.getBoundingClientRect()
      if (!rect) return
      setMenuStyle({
        position: 'fixed',
        top: rect.bottom + 4,
        right: window.innerWidth - rect.right,
      })
    }
    updatePosition()
    window.addEventListener('resize', updatePosition)
    window.addEventListener('scroll', updatePosition, true)
    return () => {
      window.removeEventListener('resize', updatePosition)
      window.removeEventListener('scroll', updatePosition, true)
    }
  }, [open])

  return (
    <div className="export-menu" ref={containerRef}>
      <button
        type="button"
        ref={buttonRef}
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
        <div
          className="export-menu-list"
          role="menu"
          style={menuStyle ?? undefined}
        >
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
