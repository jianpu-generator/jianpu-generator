import type { PartMeasureRangesOut } from 'jianpu-wasm'
import type { ByteSpan, MeasureSpan } from '../types'

/**
 * Diagnostic spans are always UTF-8 byte offsets into the Zipped source
 * (see `DiagnosticOut` in crates/jianpu-wasm/src/types.rs), but the Unzipped
 * view editor displays a different generated text. `measureSpans` (Zipped
 * source, one entry per measure) and each part's `ranges` in
 * `partMeasureRanges` (Unzipped text, one entry per measure) are index-
 * aligned by construction — both are built by walking the same score-section
 * measure grouping — so a Zipped byte offset can be relocated into the
 * Unzipped text by going through the shared measure index. Diagnostics have
 * no part/track field, so the first declared part's range for that measure
 * index is used as the anchor; every part has an entry for every measure
 * (implicit rests are filled in), so this never falls through for a
 * measure-scoped diagnostic. Returns `null` when the span doesn't fall
 * inside any measure (e.g. a `# metadata`/`# parts`/directive-line error),
 * since the Unzipped view has no equivalent position for those.
 */
export function mapZippedSpanToUnzippedRange(
  span: ByteSpan,
  measureSpans: MeasureSpan[],
  partMeasureRanges: PartMeasureRangesOut[],
): ByteSpan | null {
  const measureIndex = measureSpans.findIndex(
    (measure) => span.start >= measure.start && span.start < measure.end,
  )
  if (measureIndex === -1) return null

  const part = partMeasureRanges.find(
    (candidate) => candidate.ranges[measureIndex] !== undefined,
  )
  const range = part?.ranges[measureIndex]
  return range ? { start: range.start, end: range.end } : null
}
