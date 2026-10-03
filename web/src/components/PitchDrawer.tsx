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
  if (description.tag === 'note') {
    return (
      <div className="pitch-drawer-body">
        <div
          className="pitch-drawer-name"
          data-testid="pitch-drawer-letter-names"
        >
          {description.val.letterName}
        </div>
      </div>
    )
  }
  const chord = description.val
  return (
    <div className="pitch-drawer-body pitch-drawer-body--chord">
      <div className="pitch-drawer-text">
        <div
          className="pitch-drawer-name"
          data-testid="pitch-drawer-chord-name"
        >
          {chord.chordName}
        </div>
        <div className="pitch-drawer-detail">
          <span data-testid="pitch-drawer-letter-names">
            {chord.toneNames.join(' ')}
          </span>
        </div>
        {chord.bassNote === undefined ? null : (
          <div className="pitch-drawer-detail">
            Bass:{' '}
            <span data-testid="pitch-drawer-bass-note">{chord.bassNote}</span>
          </div>
        )}
      </div>
      <div className="pitch-drawer-diagrams">
        <PianoDiagram svg={chord.pianoDiagramSvg} />
        {chord.guitarDiagramSvg === undefined ? null : (
          <div
            className="pitch-drawer-guitar-diagram"
            data-testid="pitch-drawer-guitar-diagram"
            // Our own Rust-generated SVG (`guitar_diagram_svg.rs`), not user input.
            // biome-ignore lint/security/noDangerouslySetInnerHtml: see above
            dangerouslySetInnerHTML={{ __html: chord.guitarDiagramSvg }}
          />
        )}
      </div>
    </div>
  )
}

/** The keyboard SVG plus a caption naming the function (root, minor 3rd, ...)
 * of the highlighted key under the mouse or last pressed. Mouse hover also
 * gets the SVG's native `<title>` tooltip; the caption is what touch sees. */
function PianoDiagram({ svg }: { svg: string }) {
  const [toneFunction, setToneFunction] = useState<string | null>(null)
  const toneFunctionAt = (target: EventTarget) =>
    target instanceof Element
      ? (target
          .closest('[data-tone-function]')
          ?.getAttribute('data-tone-function') ?? null)
      : null
  return (
    <div className="pitch-drawer-piano">
      <div
        className="pitch-drawer-piano-diagram"
        data-testid="pitch-drawer-piano-diagram"
        onPointerOver={(event) => setToneFunction(toneFunctionAt(event.target))}
        onPointerDown={(event) => setToneFunction(toneFunctionAt(event.target))}
        onPointerLeave={(event) => {
          // Touch has no hover: keep the pressed tone's name shown.
          if (event.pointerType === 'mouse') setToneFunction(null)
        }}
        // Our own Rust-generated SVG (`piano_diagram_svg.rs`), not user input.
        // biome-ignore lint/security/noDangerouslySetInnerHtml: see above
        dangerouslySetInnerHTML={{ __html: svg }}
      />
      <div
        className="pitch-drawer-piano-function"
        data-testid="pitch-drawer-piano-function"
      >
        {toneFunction}
      </div>
    </div>
  )
}
