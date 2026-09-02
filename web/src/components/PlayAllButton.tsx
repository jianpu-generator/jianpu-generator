import { PauseIcon, PlayIcon } from './icons/PlaybackIcons'

interface PlayAllButtonProps {
  disabled: boolean
  loading: boolean
  playing: boolean
  onClick: () => void
  onPause: () => void
}

export function PlayAllButton({
  disabled,
  loading,
  playing,
  onClick,
  onPause,
}: PlayAllButtonProps) {
  if (playing) {
    return (
      <div className="play-measure-wrapper">
        <button
          type="button"
          className="play-all-btn play-all-btn--playing"
          data-testid="play-all-button"
          onClick={onPause}
          aria-label="Pause playback"
        >
          <PauseIcon className="play-btn-icon" /> All
        </button>
        <div className="play-measure-tooltip" role="tooltip">
          <span className="play-measure-tooltip-text">Pause playback</span>
        </div>
      </div>
    )
  }

  return (
    <div className="play-measure-wrapper">
      <button
        type="button"
        className="play-all-btn"
        data-testid="play-all-button"
        disabled={disabled}
        onClick={onClick}
        aria-label="Play entire score"
      >
        {loading ? (
          <span className="play-measure-spinner" aria-hidden="true" />
        ) : (
          <>
            <PlayIcon className="play-btn-icon" /> All
          </>
        )}
      </button>
      <div className="play-measure-tooltip" role="tooltip">
        <span className="play-measure-tooltip-text">Play entire score</span>
      </div>
    </div>
  )
}
