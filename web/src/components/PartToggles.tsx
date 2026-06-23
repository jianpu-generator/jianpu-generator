import * as Tooltip from '@radix-ui/react-tooltip'
import { Eye, EyeOff, Headphones, Mic } from 'lucide-react'
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
  loading?: boolean
}

export function PartToggles({
  parts,
  disabledParts,
  disabledLyrics,
  soloedParts,
  onPartToggle,
  onLyricsToggle,
  onSoloToggle,
  loading = false,
}: PartTogglesProps) {
  if (parts.length === 0) {
    return null
  }

  return (
    <Tooltip.Provider delayDuration={400}>
      <fieldset className="part-toggles">
        <legend className="part-toggles-label">Parts</legend>
        {loading ? (
          <span className="part-toggles-status">Updating…</span>
        ) : null}
        <ul className="part-toggles-list">
          {parts.map((part) => {
            const enabled = !disabledParts.has(part.abbreviation)
            const lyricsEnabled = !disabledLyrics.has(part.abbreviation)
            const soloed = soloedParts.has(part.abbreviation)

            return (
              <li key={part.abbreviation}>
                <div className="part-toggle-pill">
                  <span className="part-toggle-abbr">{part.abbreviation}</span>

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
      </fieldset>
    </Tooltip.Provider>
  )
}
