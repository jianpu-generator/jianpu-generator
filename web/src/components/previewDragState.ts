import type { DragPoint } from './previewDragHighlights'
import type { MeasureRange, NoteCell } from './previewSelection'

// One discriminated ref rather than a separate measure-drag and note-drag
// ref: a single mousedown can only ever arm one mode (see
// `usePreviewDragSelection`'s handler), and two independent refs risk both
// firing at once. This only governs which mode a mousedown *arms* — it
// doesn't stop a single armed mode's move/up handlers from resolving cells
// of more than one type. Both 'note' and 'lyric' mode union in the other
// cell type's marquee hits (via `applyNoteDragHighlights`/
// `applyLyricDragHighlights`, which are stateless pure functions over the
// note/lyric specs, not stateful refs), the same way 'measure' mode always
// has.
//
// A mousedown that lands on a note doesn't commit to 'note' mode right
// away — it starts 'pending' instead, and only arms into 'note' once the
// pointer has actually moved past `NOTE_DRAG_ARM_THRESHOLD_PX` (see
// `usePreviewDragSelection`'s `handleMouseMove`). A plain click on a note
// (mouseup with no meaningful movement) resolves to just that one
// note/chord cell (`noteCellAtAnchor`) — whole-measure selection is a
// separate gesture entirely now, only reachable by holding Cmd/Ctrl at
// mousedown (see `Preview.tsx`'s `onMouseDown`, which arms 'measure'
// directly for that case rather than going through 'pending' at all).
export type PreviewDragState =
  | { mode: 'measure'; anchor: MeasureRange; current: MeasureRange }
  | { mode: 'note'; anchor: DragPoint; current: DragPoint }
  | {
      mode: 'part-label'
      anchor: DragPoint
      current: DragPoint
      anchorSystem: { measureIndexStart: number; measureIndexEnd: number }
    }
  | {
      // The lyric-label mirror of 'part-label' above — a mousedown on a
      // verse row's own label (e.g. "M:v1") arms this instead, scoped to
      // its own system the same way, but resolving only that one verse's
      // syllables rather than a whole part's notes.
      mode: 'lyric-label'
      anchor: DragPoint
      current: DragPoint
      anchorSystem: { measureIndexStart: number; measureIndexEnd: number }
    }
  | {
      mode: 'pending'
      anchor: DragPoint
      noteCellAtAnchor: NoteCell
    }
  | {
      // No 'pending'-style click/drag distinction needed here, unlike
      // 'note': a lyric syllable's click target is already exactly one grid
      // column (see `LyricClickTarget`), so a plain click naturally
      // resolves to just that one syllable via the marquee test below with
      // zero movement — there's no "expand to the whole measure" shortcut
      // to arm into, unlike a note click.
      mode: 'lyric'
      anchor: DragPoint
      current: DragPoint
    }
  | null
