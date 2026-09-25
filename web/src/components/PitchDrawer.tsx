import { useRef, useState } from 'react'
import type { DescribedSelection } from '../hooks/useSelectionPitchDescription'
import type { PitchDescription } from '../types'
import { usePitchDrawerDrag } from './usePitchDrawerDrag'
import './PitchDrawer.css'

/** Bottom drawer showing the selected note/chord in letter names (and a
 * guitar chord chart for chords). Everything shown comes from
 * `pitch_description::describe_selection`; this only presents it. Dragging
 * it closed hides it until the next selection, without clearing the
 * current one. */
export function PitchDrawer({
  described,
}: {
  described: DescribedSelection | null
}) {
  const drawerRef = useRef<HTMLDivElement>(null)
  const [dismissedId, setDismissedId] = useState<number | null>(null)
  const open = described !== null && described.id !== dismissedId
  // Keeps the last description rendered while the drawer slides away, so it
  // doesn't empty out mid-animation.
  const shownRef = useRef<PitchDescription | null>(null)
  if (described !== null) shownRef.current = described.description
  const shown = shownRef.current

  const { dragOffset, handleProps } = usePitchDrawerDrag(drawerRef, () =>
    setDismissedId(described?.id ?? null),
  )

  return (
    <div
      ref={drawerRef}
      className="pitch-drawer"
      data-testid="pitch-drawer"
      data-state={open ? 'open' : 'closed'}
      data-dragging={dragOffset === null ? undefined : ''}
      aria-hidden={!open}
      style={
        open && dragOffset !== null
          ? { transform: `translateY(${dragOffset}px)` }
          : undefined
      }
    >
      <div
        className="pitch-drawer-handle"
        data-testid="pitch-drawer-handle"
        {...handleProps}
      >
        <span className="pitch-drawer-handle-bar" />
      </div>
      {shown === null ? null : <PitchDrawerContent description={shown} />}
    </div>
  )
}

function PitchDrawerContent({
  description,
}: {
  description: PitchDescription
}) {
  if (description.kind === 'note') {
    return (
      <div className="pitch-drawer-body">
        <div
          className="pitch-drawer-name"
          data-testid="pitch-drawer-letter-names"
        >
          {description.letterName}
        </div>
      </div>
    )
  }
  return (
    <div className="pitch-drawer-body">
      <div className="pitch-drawer-text">
        <div
          className="pitch-drawer-name"
          data-testid="pitch-drawer-chord-name"
        >
          {description.chordName}
        </div>
        <div className="pitch-drawer-detail">
          Notes:{' '}
          <span data-testid="pitch-drawer-letter-names">
            {description.toneNames.join(' ')}
          </span>
        </div>
        {description.bassNote === null ? null : (
          <div className="pitch-drawer-detail">
            Bass:{' '}
            <span data-testid="pitch-drawer-bass-note">
              {description.bassNote}
            </span>
          </div>
        )}
      </div>
      {description.guitarDiagramSvg === null ? null : (
        <div
          className="pitch-drawer-guitar-diagram"
          data-testid="pitch-drawer-guitar-diagram"
          // Our own Rust-generated SVG (`guitar_diagram_svg.rs`), not user input.
          // biome-ignore lint/security/noDangerouslySetInnerHtml: see above
          dangerouslySetInnerHTML={{ __html: description.guitarDiagramSvg }}
        />
      )}
    </div>
  )
}
