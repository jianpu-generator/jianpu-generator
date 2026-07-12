interface PlayFromCurrentMeasureButtonProps {
  disabled: boolean
  loading: boolean
  playing: boolean
  currentMeasure: number | null
  onClick: () => void
  onPause: () => void
  shortcutLabel: string
}

function ShortcutKeys({ label }: { label: string }) {
  const keys = label.includes('+') ? label.split('+') : [...label]
  return (
    <span className="play-measure-shortcut-keys">
      {keys.map((key, index) => (
        <span key={key}>
          {index > 0 && <span className="play-measure-shortcut-sep">+</span>}
          <kbd className="play-measure-kbd">{key}</kbd>
        </span>
      ))}
    </span>
  )
}

function Tooltip({
  shortcutLabel,
  text,
}: {
  shortcutLabel: string
  text: string
}) {
  return (
    <div className="play-measure-tooltip" role="tooltip">
      <span className="play-measure-tooltip-text">{text}</span>
      <ShortcutKeys label={shortcutLabel} />
    </div>
  )
}

export function PlayFromCurrentMeasureButton({
  disabled,
  loading,
  playing,
  currentMeasure,
  onClick,
  onPause,
  shortcutLabel,
}: PlayFromCurrentMeasureButtonProps) {
  const label = currentMeasure !== null ? `Measure ${currentMeasure + 1}` : null

  if (playing) {
    return (
      <div className="play-measure-wrapper">
        <button
          type="button"
          className="play-from-measure-btn play-from-measure-btn--playing"
          data-testid="play-from-current-measure-button"
          onClick={onPause}
          aria-label="Pause playback"
        >
          ⏸ →⏭
        </button>
        <Tooltip shortcutLabel={shortcutLabel} text="Pause playback" />
      </div>
    )
  }

  return (
    <div className="play-measure-wrapper">
      <button
        type="button"
        className="play-from-measure-btn"
        data-testid="play-from-current-measure-button"
        disabled={disabled}
        onClick={onClick}
        aria-label={
          label
            ? `Play from ${label} to end`
            : 'Play from current measure to end'
        }
      >
        {loading ? (
          <span className="play-measure-spinner" aria-hidden="true" />
        ) : (
          '▶⏭'
        )}
      </button>
      <Tooltip
        shortcutLabel={shortcutLabel}
        text={
          currentMeasure === null
            ? 'Move cursor into a measure to enable'
            : 'Play from current measure to end'
        }
      />
    </div>
  )
}
