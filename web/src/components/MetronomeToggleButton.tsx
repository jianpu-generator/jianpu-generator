import { MetronomeIcon } from './icons/PlaybackIcons'

interface MetronomeToggleButtonProps {
  enabled: boolean
  disabled: boolean
  onChange: (enabled: boolean) => void
}

export function MetronomeToggleButton({
  enabled,
  disabled,
  onChange,
}: MetronomeToggleButtonProps) {
  return (
    <div className="play-measure-wrapper">
      <button
        type="button"
        className={`metronome-toggle-btn${enabled ? ' metronome-toggle-btn--on' : ''}`}
        data-testid="metronome-toggle"
        aria-pressed={enabled}
        disabled={disabled}
        onClick={() => onChange(!enabled)}
        aria-label={enabled ? 'Metronome on' : 'Metronome off'}
      >
        <MetronomeIcon className="play-btn-icon" /> Tick
      </button>
      <div className="play-measure-tooltip" role="tooltip">
        <span>{enabled ? 'Metronome: on' : 'Metronome: off'}</span>
        <span className="play-measure-tooltip-text">
          Click on every beat while playing
        </span>
      </div>
    </div>
  )
}
