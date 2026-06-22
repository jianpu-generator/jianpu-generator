import { Eye, EyeOff, Headphones } from 'lucide-react'
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
    <fieldset className="part-toggles">
      <legend className="part-toggles-label">Parts</legend>
      {loading ? <span className="part-toggles-status">Updating…</span> : null}
      <ul className="part-toggles-list">
        {parts.map((part) => {
          const enabled = !disabledParts.has(part.abbreviation)
          const lyricsEnabled = !disabledLyrics.has(part.abbreviation)
          const soloed = soloedParts.has(part.abbreviation)
          const title =
            part.display_name === part.abbreviation
              ? part.abbreviation
              : `${part.display_name} (${part.abbreviation})`

          return (
            <li key={part.abbreviation} className="part-toggle-group">
              <label className="part-toggle part-toggle--icon" title={title}>
                <input
                  type="checkbox"
                  checked={enabled}
                  onChange={(event) =>
                    onPartToggle(part.abbreviation, event.target.checked)
                  }
                />
                {enabled ? (
                  <Eye size={14} aria-hidden="true" />
                ) : (
                  <EyeOff size={14} aria-hidden="true" />
                )}
                <span className="part-toggle-label">{part.abbreviation}</span>
              </label>
              <label
                className="part-toggle part-toggle--solo"
                title={`Solo ${title}`}
              >
                <input
                  type="checkbox"
                  checked={soloed}
                  onChange={(event) =>
                    onSoloToggle(part.abbreviation, event.target.checked)
                  }
                />
                <Headphones size={14} aria-hidden="true" />
              </label>
              {part.has_lyrics && enabled ? (
                <>
                  <span className="part-toggle-connector" aria-hidden="true" />
                  <label
                    className="part-toggle part-toggle--lyrics"
                    title={`${title} lyrics`}
                  >
                    <input
                      type="checkbox"
                      checked={lyricsEnabled}
                      onChange={(event) =>
                        onLyricsToggle(part.abbreviation, event.target.checked)
                      }
                    />
                    <span className="part-toggle-label">lyrics</span>
                  </label>
                </>
              ) : null}
            </li>
          )
        })}
      </ul>
    </fieldset>
  )
}
