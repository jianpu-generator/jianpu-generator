import type { Notation } from '../hooks/useStaffNotation'

interface NotationToggleProps {
  notation: Notation
  onChange: (notation: Notation) => void
}

const OPTIONS: { value: Notation; icon: string; label: string }[] = [
  { value: 'jianpu', icon: '簡', label: 'Jianpu' },
  { value: 'staff', icon: '𝄞', label: 'Staff' },
]

/** Switches the preview between jianpu and western staff notation. */
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
          <span className="notation-toggle-icon" aria-hidden="true">
            {icon}
          </span>
          {label}
        </button>
      ))}
    </fieldset>
  )
}
