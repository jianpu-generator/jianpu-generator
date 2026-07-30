import * as Tooltip from '@radix-ui/react-tooltip'
import {
  ChevronDown,
  ChevronRight,
  Eye,
  EyeOff,
  Headphones,
  Mic,
} from 'lucide-react'
import { useState } from 'react'
import type { PartInfo } from '../types'
import './PartToggles.css'

interface PartTogglesProps {
  parts: PartInfo[]
  disabledParts: ReadonlySet<string>
  disabledLyrics: ReadonlySet<string>
  soloedParts: ReadonlySet<string>
  onPartToggle: (abbreviation: string, enabled: boolean) => void
  onLyricsToggle: (abbreviation: string, enabled: boolean) => void
  onSoloToggle: (abbreviation: string, soloed: boolean) => void
}

export function PartToggles({
  parts,
  disabledParts,
  disabledLyrics,
  soloedParts,
  onPartToggle,
  onLyricsToggle,
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
            {parts.map((part) => {
              const enabled = !disabledParts.has(part.abbreviation)
              const lyricsEnabled = !disabledLyrics.has(part.abbreviation)
              const soloed = soloedParts.has(part.abbreviation)

              return (
                <li key={part.abbreviation}>
                  <div className="part-toggle-pill">
                    <Tooltip.Root>
                      <Tooltip.Trigger asChild>
                        <span className="part-toggle-abbr">
                          {part.abbreviation}
                        </span>
                      </Tooltip.Trigger>
                      <Tooltip.Portal>
                        <Tooltip.Content
                          className="part-toggle-tooltip-content"
                          sideOffset={4}
                        >
                          {part.display_name}
                        </Tooltip.Content>
                      </Tooltip.Portal>
                    </Tooltip.Root>

                    <Tooltip.Root>
                      <Tooltip.Trigger asChild>
                        <label className="part-toggle-segment part-toggle-segment--eye">
                          <input
                            type="checkbox"
                            checked={enabled}
                            onChange={(event) =>
                              onPartToggle(
                                part.abbreviation,
                                event.target.checked,
                              )
                            }
                          />
                          {enabled ? (
                            <Eye size={14} aria-hidden="true" />
                          ) : (
                            <EyeOff size={14} aria-hidden="true" />
                          )}
                        </label>
                      </Tooltip.Trigger>
                      <Tooltip.Portal>
                        <Tooltip.Content
                          className="part-toggle-tooltip-content"
                          sideOffset={4}
                        >
                          Show/Hide
                        </Tooltip.Content>
                      </Tooltip.Portal>
                    </Tooltip.Root>

                    <Tooltip.Root>
                      <Tooltip.Trigger asChild>
                        <label className="part-toggle-segment part-toggle-segment--headphones">
                          <input
                            type="checkbox"
                            checked={soloed}
                            onChange={(event) =>
                              onSoloToggle(
                                part.abbreviation,
                                event.target.checked,
                              )
                            }
                          />
                          <Headphones size={14} aria-hidden="true" />
                        </label>
                      </Tooltip.Trigger>
                      <Tooltip.Portal>
                        <Tooltip.Content
                          className="part-toggle-tooltip-content"
                          sideOffset={4}
                        >
                          Solo
                        </Tooltip.Content>
                      </Tooltip.Portal>
                    </Tooltip.Root>

                    {part.has_lyrics && enabled ? (
                      <Tooltip.Root>
                        <Tooltip.Trigger asChild>
                          <label className="part-toggle-segment part-toggle-segment--mic">
                            <input
                              type="checkbox"
                              checked={lyricsEnabled}
                              onChange={(event) =>
                                onLyricsToggle(
                                  part.abbreviation,
                                  event.target.checked,
                                )
                              }
                            />
                            <Mic size={14} aria-hidden="true" />
                          </label>
                        </Tooltip.Trigger>
                        <Tooltip.Portal>
                          <Tooltip.Content
                            className="part-toggle-tooltip-content"
                            sideOffset={4}
                          >
                            Lyrics
                          </Tooltip.Content>
                        </Tooltip.Portal>
                      </Tooltip.Root>
                    ) : null}
                  </div>
                </li>
              )
            })}
          </ul>
        )}
      </fieldset>
    </Tooltip.Provider>
  )
}
