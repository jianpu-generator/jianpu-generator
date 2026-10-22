import type { Notation } from '../hooks/useStaffNotation'

interface NotationToggleProps {
  notation: Notation
  onChange: (notation: Notation) => void
}

const OPTIONS: { value: Notation; label: string }[] = [
  { value: 'jianpu', label: 'Jianpu' },
  { value: 'staff', label: 'Staff' },
]

/** Switches the preview between jianpu and western staff notation. */
export function NotationToggle({ notation, onChange }: NotationToggleProps) {
  return (
    <fieldset className="notation-toggle" data-testid="notation-toggle">
      <legend className="notation-toggle-legend">Notation</legend>
      {OPTIONS.map(({ value, label }) => (
        <button
          key={value}
          type="button"
          className={`notation-toggle-btn${notation === value ? ' notation-toggle-btn--on' : ''}`}
          aria-pressed={notation === value}
          data-testid={`notation-toggle-${value}`}
          onClick={() => onChange(value)}
        >
          {label}
        </button>
      ))}
    </fieldset>
  )
}
