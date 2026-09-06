import * as DropdownMenu from '@radix-ui/react-dropdown-menu'
import { ChevronDownIcon } from '@radix-ui/react-icons'
import type { ReactNode } from 'react'

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
  return (
    <div className="export-menu">
      <DropdownMenu.Root modal={false}>
        <DropdownMenu.Trigger asChild>
          <button
            type="button"
            className="preview-export-btn"
            disabled={disabled}
          >
            {icon}
            {label}
            <ChevronDownIcon className="export-menu-caret" aria-hidden="true" />
          </button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            className="export-menu-list"
            align="end"
            sideOffset={4}
          >
            {sections.map((section, index) => (
              <div className="export-menu-section" key={section.title ?? index}>
                {section.title ? (
                  <DropdownMenu.Label className="export-menu-section-title">
                    {section.title}
                  </DropdownMenu.Label>
                ) : null}
                {section.items.map((item) => (
                  <DropdownMenu.Item
                    key={item.key}
                    className="export-menu-item"
                    disabled={item.disabled}
                    onSelect={() => item.onSelect()}
                  >
                    {item.icon}
                    {item.busy ? item.busyLabel : item.label}
                  </DropdownMenu.Item>
                ))}
              </div>
            ))}
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>
    </div>
  )
}
