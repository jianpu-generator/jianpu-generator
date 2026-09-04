import type { NoteTimingOut, PartOut } from '../jianpuWasm'

/**
 * Rust's note-timing pipeline (`note_timings_seconds`, see
 * `src/midi/timing_note_timings.rs`) takes a `visible_tracks` parameter
 * (the part-visibility toggle's own state) that it applies *before*
 * resolving each note's `source_part_index`/`note_id`, so when a caller
 * passes it, the returned `source_part_index` already lands in the same
 * hidden-parts-compacted index space the rendered SVG's `data-part-index`
 * uses (`apply_track_filter` physically removes hidden parts before
 * compiling — see `src/filters.rs`) — no further remapping needed.
 * `worker/audioMessageHandlers.ts` does exactly that, so this function is no
 * longer called from production code.
 *
 * This utility remains for the case where a caller only has `NoteTiming`s
 * computed *without* a `visible_tracks` filter (`source_part_index` still
 * the note's true *written* index) and needs them remapped into the
 * hidden-parts-compacted space after the fact — e.g. against a differently
 * fetched/cached timing set. `visibleTracks` must be the part visibility
 * toggle's current state (never a playback-only mute override) —
 * `undefined` means no part is hidden, in which case written and compacted
 * indices already agree and no remapping is needed.
 */
export function remapNoteTimingsToVisiblePartIndex(
  timings: NoteTimingOut[],
  parts: PartOut[],
  visibleTracks: string[] | undefined,
): NoteTimingOut[] {
  if (visibleTracks === undefined) return timings
  const compactedIndexByWrittenIndex = new Map<number, number>()
  let nextCompactedIndex = 0
  parts.forEach((part, writtenIndex) => {
    if (visibleTracks.includes(part.abbreviation)) {
      compactedIndexByWrittenIndex.set(writtenIndex, nextCompactedIndex)
      nextCompactedIndex += 1
    }
  })
  return timings.flatMap((timing) => {
    const compactedIndex = compactedIndexByWrittenIndex.get(
      timing.source_part_index,
    )
    return compactedIndex === undefined
      ? []
      : [{ ...timing, source_part_index: compactedIndex }]
  })
}
