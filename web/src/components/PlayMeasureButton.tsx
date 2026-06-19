interface PlayMeasureButtonProps {
  disabled: boolean
  loading: boolean
  playing: boolean
  measureRange: { start: number; end: number } | null
  onClick: () => void
  onPause: () => void
  shortcutLabel: string
}

function measureLabel(range: { start: number; end: number }): string {
  if (range.start === range.end) {
    return `Measure ${range.start + 1}`
  }
  return `Measures ${range.start + 1}–${range.end + 1}`
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

export function PlayMeasureButton({
  disabled,
  loading,
  playing,
  measureRange,
  onClick,
  onPause,
  shortcutLabel,
}: PlayMeasureButtonProps) {
  const label = measureRange !== null ? measureLabel(measureRange) : null

  if (playing) {
    return (
      <div className="play-measure-wrapper">
        <button
          type="button"
          className="play-measure-btn play-measure-btn--playing"
          onClick={onPause}
          aria-label={label ? `Pause ${label}` : 'Pause playback'}
        >
          {label ? `⏸ ${label}` : '⏸'}
        </button>
        <Tooltip shortcutLabel={shortcutLabel} text="Pause playback" />
      </div>
    )
  }

  return (
    <div className="play-measure-wrapper">
      <button
        type="button"
        className="play-measure-btn"
        disabled={disabled}
        onClick={onClick}
        aria-label={label ? `▶ ${label}` : 'Play selected measure(s)'}
      >
        {loading ? (
          <span className="play-measure-spinner" aria-hidden="true" />
        ) : label !== null ? (
          `▶ ${label}`
        ) : (
          '▶'
        )}
      </button>
      <Tooltip
        shortcutLabel={shortcutLabel}
        text={
          measureRange === null
            ? 'Move cursor into a measure to enable'
            : 'Play selected measure(s)'
        }
      />
    </div>
  )
}
