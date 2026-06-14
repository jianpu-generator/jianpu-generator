interface PlayMeasureButtonProps {
  disabled: boolean
  loading: boolean
  onClick: () => void
}

export function PlayMeasureButton({
  disabled,
  loading,
  onClick,
}: PlayMeasureButtonProps) {
  return (
    <button
      type="button"
      className="play-measure-btn"
      disabled={disabled}
      onClick={onClick}
      title={
        disabled && !loading
          ? 'Move cursor into a measure to enable'
          : 'Play current measure'
      }
      aria-label="Play current measure"
    >
      {loading ? (
        <span className="play-measure-spinner" aria-hidden="true" />
      ) : (
        '▶'
      )}
    </button>
  )
}
