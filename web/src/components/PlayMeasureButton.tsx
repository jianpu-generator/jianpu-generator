interface PlayMeasureButtonProps {
  disabled: boolean
  loading: boolean
  measureRange: { start: number; end: number } | null
  onClick: () => void
}

function measureLabel(range: { start: number; end: number }): string {
  if (range.start === range.end) {
    return `▶ Measure ${range.start + 1}`
  }
  return `▶ Measures ${range.start + 1}–${range.end + 1}`
}

export function PlayMeasureButton({
  disabled,
  loading,
  measureRange,
  onClick,
}: PlayMeasureButtonProps) {
  const label = measureRange !== null ? measureLabel(measureRange) : null
  return (
    <button
      type="button"
      className="play-measure-btn"
      disabled={disabled}
      onClick={onClick}
      title={
        measureRange === null
          ? 'Move cursor into a measure to enable'
          : 'Play selected measure(s)'
      }
      aria-label={label ?? 'Play selected measure(s)'}
    >
      {loading ? (
        <span className="play-measure-spinner" aria-hidden="true" />
      ) : label !== null ? (
        label
      ) : (
        '▶'
      )}
    </button>
  )
}
