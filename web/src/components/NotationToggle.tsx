import type { Notation } from '../hooks/useStaffNotation'

interface NotationToggleProps {
  notation: Notation
  onChange: (notation: Notation) => void
}

// ABC, Guitar and Harmonica have no icon: their labels already read on their
// own. A native <select> can only hold text, so the icon is prefixed to it.
const OPTIONS: { value: Notation; icon?: string; label: string }[] = [
  { value: 'jianpu', icon: '簡', label: '123' },
  { value: 'staff', icon: '𝄞', label: 'Staff' },
  { value: 'abc', label: 'ABC' },
  { value: 'guitar', label: 'Guitar' },
  { value: 'harmonica', label: 'Harmonica' },
]

/** Switches the preview between jianpu, western staff notation, ABC (the
 * jianpu layout with letter names in place of degree numbers), Guitar (the
 * jianpu layout with a chord diagram at each chord symbol), and Harmonica
 * (the jianpu layout with diatonic harmonica tabs). A native dropdown, so it
 * stays compact as views are added and opens the platform picker on mobile. */
export function NotationToggle({ notation, onChange }: NotationToggleProps) {
  return (
    <label className="notation-toggle" data-testid="notation-toggle">
      <span className="notation-toggle-legend">Notation</span>
      <select
        className="notation-toggle-select"
        data-testid="notation-select"
        value={notation}
        onChange={(event) => onChange(event.target.value as Notation)}
      >
        {OPTIONS.map(({ value, icon, label }) => (
          <option key={value} value={value}>
            {icon ? `${icon} ${label}` : label}
          </option>
        ))}
      </select>
    </label>
  )
}
