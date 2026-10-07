import * as Tooltip from '@radix-ui/react-tooltip'
import {
  ChevronDown,
  ChevronRight,
  Eye,
  EyeOff,
  Headphones,
} from 'lucide-react'
import { useState } from 'react'
import type { PartInfo } from '../types'
import './PartToggles.css'

interface PartTogglesProps {
  parts: PartInfo[]
  disabledParts: ReadonlySet<string>
  soloedParts: ReadonlySet<string>
  onPartToggle: (abbreviation: string, enabled: boolean) => void
  onSoloToggle: (abbreviation: string, soloed: boolean) => void
}

function TooltipLabel({
  tooltip,
  children,
}: {
  tooltip: string
  children: React.ReactNode
}) {
  return (
    <Tooltip.Root>
      <Tooltip.Trigger asChild>{children}</Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content className="part-toggle-tooltip-content" sideOffset={4}>
          {tooltip}
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  )
}

interface TogglePillProps {
  label: string
  tooltip: string
  visible: boolean
  soloed: boolean
  onVisibleChange: (visible: boolean) => void
  onSoloChange: (soloed: boolean) => void
}

/** One show/hide + solo pill: used once for a part's notes row and once for
 * its lyrics row. */
function TogglePill({
  label,
  tooltip,
  visible,
  soloed,
  onVisibleChange,
  onSoloChange,
}: TogglePillProps) {
  return (
    <div className="part-toggle-pill">
      <TooltipLabel tooltip={tooltip}>
        <span className="part-toggle-abbr">{label}</span>
      </TooltipLabel>
      <TooltipLabel tooltip="Show/Hide">
        <label className="part-toggle-segment part-toggle-segment--eye">
          <input
            type="checkbox"
            checked={visible}
            onChange={(event) => onVisibleChange(event.target.checked)}
          />
          {visible ? (
            <Eye size={14} aria-hidden="true" />
          ) : (
            <EyeOff size={14} aria-hidden="true" />
          )}
        </label>
      </TooltipLabel>
      <TooltipLabel tooltip="Solo">
        <label className="part-toggle-segment part-toggle-segment--headphones">
          <input
            type="checkbox"
            checked={soloed}
            onChange={(event) => onSoloChange(event.target.checked)}
          />
          <Headphones size={14} aria-hidden="true" />
        </label>
      </TooltipLabel>
    </div>
  )
}

export function PartToggles({
  parts,
  disabledParts,
  soloedParts,
  onPartToggle,
  onSoloToggle,
}: PartTogglesProps) {
  const [collapsed, setCollapsed] = useState(false)

  if (parts.length === 0) {
    return null
  }

  return (
    <Tooltip.Provider delayDuration={400}>
      <fieldset
        className={[
          'part-toggles',
          collapsed ? 'part-toggles--collapsed' : '',
        ].join(' ')}
      >
        <legend className="visually-hidden">Parts</legend>
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
          Parts
        </button>
        {collapsed ? null : (
          <ul className="part-toggles-list toolbar-scroll-list">
            {parts.map((part) => (
              <li key={part.abbreviation}>
                <TogglePill
                  label={part.abbreviation}
                  tooltip={part.displayName}
                  visible={!disabledParts.has(part.abbreviation)}
                  soloed={soloedParts.has(part.abbreviation)}
                  onVisibleChange={(visible) =>
                    onPartToggle(part.abbreviation, visible)
                  }
                  onSoloChange={(soloed) =>
                    onSoloToggle(part.abbreviation, soloed)
                  }
                />
              </li>
            ))}
          </ul>
        )}
      </fieldset>
    </Tooltip.Provider>
  )
}
