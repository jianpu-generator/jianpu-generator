import type { Notation } from '../hooks/useStaffNotation'

interface NotationToggleProps {
  notation: Notation
  onChange: (notation: Notation) => void
}

// ABC has no icon: its label already reads as its own symbol.
const OPTIONS: { value: Notation; icon?: string; label: string }[] = [
  { value: 'jianpu', icon: '簡', label: '123' },
  { value: 'staff', icon: '𝄞', label: 'Staff' },
  { value: 'abc', label: 'ABC' },
]

/** Switches the preview between jianpu, western staff notation, and ABC (the
 * jianpu layout with letter names in place of degree numbers). */
export function NotationToggle({ notation, onChange }: NotationToggleProps) {
  return (
    <fieldset className="notation-toggle" data-testid="notation-toggle">
      <legend className="notation-toggle-legend">Notation</legend>
      {OPTIONS.map(({ value, icon, label }) => (
        <button
          key={value}
          type="button"
          className={`notation-toggle-btn${notation === value ? ' notation-toggle-btn--on' : ''}`}
          aria-pressed={notation === value}
          data-testid={`notation-toggle-${value}`}
          onClick={() => onChange(value)}
        >
          {icon && (
            <span className="notation-toggle-icon" aria-hidden="true">
              {icon}
            </span>
          )}
          {label}
        </button>
      ))}
    </fieldset>
  )
}
