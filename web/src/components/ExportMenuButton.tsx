import { ChevronDownIcon } from '@radix-ui/react-icons'
import type { ReactNode } from 'react'
import { ResponsiveMenu } from './ResponsiveMenu'

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
      <ResponsiveMenu.Root>
        <ResponsiveMenu.Trigger asChild>
          <button
            type="button"
            className="preview-export-btn"
            disabled={disabled}
          >
            {icon}
            {label}
            <ChevronDownIcon className="export-menu-caret" aria-hidden="true" />
          </button>
        </ResponsiveMenu.Trigger>
        <ResponsiveMenu.Content
          className="export-menu-list"
          align="end"
          sideOffset={4}
          title={label}
        >
          {sections.map((section, index) => (
            <div className="export-menu-section" key={section.title ?? index}>
              {section.title ? (
                <ResponsiveMenu.Label className="export-menu-section-title">
                  {section.title}
                </ResponsiveMenu.Label>
              ) : null}
              {section.items.map((item) => (
                <ResponsiveMenu.Item
                  key={item.key}
                  className="export-menu-item"
                  disabled={item.disabled}
                  onSelect={() => item.onSelect()}
                >
                  {item.icon}
                  {item.busy ? item.busyLabel : item.label}
                </ResponsiveMenu.Item>
              ))}
            </div>
          ))}
        </ResponsiveMenu.Content>
      </ResponsiveMenu.Root>
    </div>
  )
}
