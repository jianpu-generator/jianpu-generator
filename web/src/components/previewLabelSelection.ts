import type { LyricLabelElementId, PartLabelElementId } from '../jianpuWasm'
import type { LyricSpan, NoteSpan } from '../types'
import {
  getClickableElementIdAtPoint,
  type LyricCell,
  type NoteCell,
} from './previewSelection'

/** One rendered part-label click target, keyed the same way as
 * `Tag` `part-label`'s `data-part-index`/`data-measure-index-start`/
 * `data-measure-index-end` SVG attributes — see `getPartLabelAtPoint`. */
export type PartLabelHit = PartLabelElementId

/** The part-label click target under the given point, if any — reads the
 * invisible `PartLabelClickTarget` rect's enclosing `Tag` `part-label` group
 * (see `renderer::new_renderer::render_part_label_click_target`). */
export function getPartLabelAtPoint(
  x: number,
  y: number,
): PartLabelHit | undefined {
  const id = getClickableElementIdAtPoint(x, y, 'part-label')
  return id?.tag === 'part-label' ? id.val : undefined
}

/** Every note/rest cell belonging to the given part-label hits — each hit
 * selects its own part's notes across its own `measureIndexStart..=measureIndexEnd`
 * (the whole system the label sits in), mirroring `noteCellsInMeasureRange`. */
export function noteCellsForPartLabels(
  noteSpans: NoteSpan[],
  hits: PartLabelHit[],
): NoteCell[] {
  return hits.flatMap((hit) =>
    noteSpans
      .filter(
        (span) =>
          span.sourcePartIndex === hit.sourcePartIndex &&
          span.measureIndex >= hit.measureIndexStart &&
          span.measureIndex <= hit.measureIndexEnd,
      )
      .map((span) => ({
        sourcePartIndex: span.sourcePartIndex,
        noteId: span.noteId,
      })),
  )
}

/** Every lyric syllable cell belonging to the given part-label hits —
 * the lyric-side mirror of `noteCellsForPartLabels`, so a part-label
 * range-select selects the verse lyrics under its swept part rows alongside
 * their notes. */
export function lyricCellsForPartLabels(
  lyricSpans: LyricSpan[],
  hits: PartLabelHit[],
): LyricCell[] {
  return hits.flatMap((hit) =>
    lyricSpans
      .filter(
        (span) =>
          // A part's lyric parts travel with it.
          (span.sourcePartIndex === hit.sourcePartIndex ||
            span.targetSourcePartIndex === hit.sourcePartIndex) &&
          span.measureIndex >= hit.measureIndexStart &&
          span.measureIndex <= hit.measureIndexEnd,
      )
      .map((span) => ({
        sourcePartIndex: span.sourcePartIndex,
        noteId: span.noteId,
      })),
  )
}

/** One rendered lyric-label click target, keyed the same way as
 * `Tag::LyricLabel`'s `data-part-index`/
 * `data-measure-index-start`/`data-measure-index-end` SVG attributes — see
 * `getLyricLabelAtPoint`. The lyric-side mirror of `PartLabelHit`. */
export type LyricLabelHit = LyricLabelElementId

/** The lyric-label click target under the given point, if any — reads the
 * invisible `LyricLabelClickTarget` rect's enclosing `Tag::LyricLabel` group
 * (see `renderer::new_renderer::render_lyric_label_click_target`). The
 * lyric-side mirror of `getPartLabelAtPoint`. */
export function getLyricLabelAtPoint(
  x: number,
  y: number,
): LyricLabelHit | undefined {
  const id = getClickableElementIdAtPoint(x, y, 'lyric-label')
  return id?.tag === 'lyric-label' ? id.val : undefined
}

/** Every lyric syllable cell belonging to the given lyric-label hits — each
 * hit selects its own verse's syllables across its own
 * `measureIndexStart..=measureIndexEnd` (the whole system the label sits
 * in), the lyric-side mirror of `noteCellsForPartLabels`. */
export function lyricCellsForLyricLabels(
  lyricSpans: LyricSpan[],
  hits: LyricLabelHit[],
): LyricCell[] {
  return hits.flatMap((hit) =>
    lyricSpans
      .filter(
        (span) =>
          span.sourcePartIndex === hit.sourcePartIndex &&
          span.measureIndex >= hit.measureIndexStart &&
          span.measureIndex <= hit.measureIndexEnd,
      )
      .map((span) => ({
        sourcePartIndex: span.sourcePartIndex,
        noteId: span.noteId,
      })),
  )
}
