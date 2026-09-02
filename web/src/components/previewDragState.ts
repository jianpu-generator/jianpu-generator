import type { DragPoint } from './previewDragHighlights'
import type { MeasureRange, NoteCell } from './previewSelection'

// One discriminated ref rather than a separate measure-select and note-select
// ref: a single click can only ever anchor one mode (see
// `handlePreviewClick`'s idle-click branch), and two independent refs risk
// both firing at once. This only governs which mode a click *anchors* — it
// doesn't stop a single anchored mode's hover/second-click resolution from
// resolving cells of more than one type. Both 'note' and 'lyric' mode union
// in the other cell type's marquee hits (via `applyNoteDragHighlights`/
// `applyLyricDragHighlights`, which are stateless pure functions over the
// note/lyric specs, not stateful refs), the same way 'measure' mode always
// has.
//
// Selection is a click-and-click gesture, not a held-button drag: a first
// click anchors one of the modes below and eagerly highlights whatever it
// landed on; a plain `mousemove` (no button held) live-updates the
// anchor→hover marquee for mouse users; a second click resolves the range
// between the anchor and wherever that click landed and commits it (see
// `handlePreviewClick`). This is what lets the same gesture work for touch —
// a tap synthesizes a `click` with no intervening movement, so two taps are
// just two clicks with the marquee collapsing to whatever's under the second
// tap.
//
// A click on a note doesn't resolve to just that one note/chord cell right
// away — it anchors 'note' mode with `noteCellAtAnchor` recorded alongside
// the live `anchor`/`current` points, so the second click can either widen
// into a real marquee (a different target) or, if it misses every note's
// click target (e.g. it landed back on the same note, or on a bar-line/
// gutter pixel), fall back to `noteCellAtAnchor` rather than resolving to
// nothing — see `handlePreviewClick`'s 'note' commit branch. Whole-measure
// selection off a note/lyric/gutter pixel is a separate gesture entirely,
// only reachable by holding Cmd/Ctrl at the first click (see
// `previewClickHandler.ts`, which anchors 'measure' directly for that case
// rather than going through 'note' at all). Landing exactly on a bar line's
// own divider is the one exception: it always anchors 'measure'
// unconditionally, no modifier needed (see that same file's bar-line-handle
// check) — grabbing the divider itself is unambiguous in a way clicking a
// note or empty gutter isn't.
export type PreviewDragState =
  | { mode: 'measure'; anchor: MeasureRange; current: MeasureRange }
  | {
      mode: 'note'
      anchor: DragPoint
      current: DragPoint
      /** The note/chord cell the anchoring click landed on (or the nearest
       * one in its measure, for the gutter-miss fallback) — used to resolve
       * the second click when it doesn't land on any note's own click
       * target, so the gesture still collapses to a single cell instead of
       * an empty selection. See this type's doc comment above. */
      noteCellAtAnchor: NoteCell
    }
  | {
      mode: 'part-label'
      anchor: DragPoint
      current: DragPoint
      anchorSystem: { measureIndexStart: number; measureIndexEnd: number }
    }
  | {
      // Cmd/Ctrl-click on a part label — the label-side mirror of 'measure'
      // mode's Cmd/Ctrl gate above. Elevates 'part-label' mode's granularity
      // from "one part, one system" to "every part in every system the
      // gesture touches": a bare click resolves to the whole system the
      // clicked label sits in, and a second click further away sweeps in
      // whole additional systems as it touches their label rows (see
      // `partLabelsInMarqueeAcrossSystems`). No `anchorSystem` needed here,
      // unlike 'part-label' — this mode is deliberately unrestricted to any
      // one system.
      mode: 'part-label-system'
      anchor: DragPoint
      current: DragPoint
    }
  | {
      // The lyric-label mirror of 'part-label' above — a click on a verse
      // row's own label (e.g. "M:v1") anchors this instead, scoped to its
      // own system the same way, but resolving only that one verse's
      // syllables rather than a whole part's notes.
      mode: 'lyric-label'
      anchor: DragPoint
      current: DragPoint
      anchorSystem: { measureIndexStart: number; measureIndexEnd: number }
    }
  | {
      // No note-style single-cell fallback needed here: a lyric syllable's
      // click target is already exactly one grid column (see
      // `LyricClickTarget`), so a second click with no meaningful movement
      // naturally resolves to just that one syllable via the marquee test
      // below with zero movement — there's no "expand to the whole measure"
      // shortcut to anchor into, unlike a note click.
      mode: 'lyric'
      anchor: DragPoint
      current: DragPoint
    }
  | null
