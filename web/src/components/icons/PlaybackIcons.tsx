/** Solid (filled) playback glyphs.
 *
 * `@radix-ui/react-icons`'s PlayIcon/PauseIcon/TrackNextIcon render as
 * hollow outlines (an evenodd double-path trick), matching that set's
 * line-icon style. Radix has no filled variant for these, so these are
 * small hand-drawn solid replacements on the same 15x15 grid.
 */

interface PlaybackIconProps {
  className?: string
}

export function PlayIcon({ className }: PlaybackIconProps) {
  return (
    <svg
      className={className}
      width="15"
      height="15"
      viewBox="0 0 15 15"
      fill="currentColor"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <path d="M3.5 2.25C3.5 1.87 3.92 1.64 4.24 1.85L12.24 7.1C12.53 7.29 12.53 7.71 12.24 7.9L4.24 13.15C3.92 13.36 3.5 13.13 3.5 12.75V2.25Z" />
    </svg>
  )
}

export function PauseIcon({ className }: PlaybackIconProps) {
  return (
    <svg
      className={className}
      width="15"
      height="15"
      viewBox="0 0 15 15"
      fill="currentColor"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <rect x="3.5" y="2" width="3" height="11" rx="1" />
      <rect x="8.5" y="2" width="3" height="11" rx="1" />
    </svg>
  )
}

export function TrackNextIcon({ className }: PlaybackIconProps) {
  return (
    <svg
      className={className}
      width="15"
      height="15"
      viewBox="0 0 15 15"
      fill="currentColor"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <path d="M2.5 2.25C2.5 1.87 2.92 1.64 3.24 1.85L10.5 6.5V2.5C10.5 2.22 10.72 2 11 2H11.5C11.78 2 12 2.22 12 2.5V12.5C12 12.78 11.78 13 11.5 13H11C10.72 13 10.5 12.78 10.5 12.5V8.5L3.24 13.15C2.92 13.36 2.5 13.13 2.5 12.75V2.25Z" />
    </svg>
  )
}
