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
      <button
        type="button"
        className="play-measure-btn play-measure-btn--playing"
        onClick={onPause}
        title={`Pause playback (${shortcutLabel})`}
        aria-label={label ? `Pause ${label}` : 'Pause playback'}
      >
        {label ? `⏸ ${label}` : '⏸'}
      </button>
    )
  }

  return (
    <button
      type="button"
      className="play-measure-btn"
      disabled={disabled}
      onClick={onClick}
      title={
        measureRange === null
          ? 'Move cursor into a measure to enable'
          : `Play selected measure(s) (${shortcutLabel})`
      }
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
  )
}
