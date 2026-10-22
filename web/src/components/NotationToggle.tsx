import type { Notation } from '../hooks/useStaffNotation'

interface NotationToggleProps {
  notation: Notation
  onChange: (notation: Notation) => void
}

// ABC, Guitar and Harmonica have no icon: their labels already read on their
// own.
const OPTIONS: { value: Notation; icon?: string; label: string }[] = [
  { value: 'jianpu', icon: '簡', label: '123' },
  { value: 'staff', icon: '𝄞', label: 'Staff' },
  { value: 'abc', label: 'ABC' },
  { value: 'guitar', label: 'Guitar' },
  { value: 'harmonica', label: 'Harmonica' },
]

/** Switches the preview between jianpu, western staff notation, ABC (the
 * jianpu layout with letter names in place of degree numbers), Guitar (the
 * jianpu layout with each degree followed by its letter name, `1 (C)`, and
 * a chord diagram at each chord symbol), and Harmonica
 * (the jianpu layout with diatonic harmonica tabs). */
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
