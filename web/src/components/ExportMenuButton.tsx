import { ChevronDownIcon } from '@radix-ui/react-icons'
import type { ReactNode } from 'react'
import { useRef } from 'react'
import { useDismissableOpen } from '../hooks/useDismissableOpen'
import { useFixedMenuPosition } from '../hooks/useFixedMenuPosition'
import { FixedMenuPortal } from './FixedMenuPortal'

export interface ExportMenuItem {
  key: string
  label: string
  busyLabel: string
  busy: boolean
  disabled: boolean
  onSelect: () => void
  icon?: ReactNode
}

export interface ExportMenuSection {
  /** Heading shown above the section's items. Omit for an unlabelled group
   * (e.g. the first section in a menu that needs no introduction). */
  title?: string
  items: ExportMenuItem[]
}

interface ExportMenuButtonProps {
  label: string
  icon?: ReactNode
  sections: ExportMenuSection[]
  disabled?: boolean
}

export function ExportMenuButton({
  label,
  icon,
  sections,
  disabled = false,
}: ExportMenuButtonProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const buttonRef = useRef<HTMLButtonElement>(null)
  const menuRef = useRef<HTMLDivElement>(null)
  const [open, setOpen] = useDismissableOpen(containerRef, menuRef)
  const menuStyle = useFixedMenuPosition(buttonRef, open)

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
        <FixedMenuPortal>
          <div
            className="export-menu-list"
            role="menu"
            style={menuStyle}
            ref={menuRef}
          >
            {sections.map((section, index) => (
              <div className="export-menu-section" key={section.title ?? index}>
                {section.title ? (
                  <div
                    className="export-menu-section-title"
                    role="presentation"
                  >
                    {section.title}
                  </div>
                ) : null}
                {section.items.map((item) => (
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
            ))}
          </div>
        </FixedMenuPortal>
      ) : null}
    </div>
  )
}
