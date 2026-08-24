import type { NoteTimingOut, PartOut } from 'jianpu-wasm'

/**
 * Rust's note-timing pipeline (`note_timings_seconds`, see
 * `src/midi/timing_note_timings.rs`) deliberately reports each
 * `NoteTiming.source_part_index` as the note's true *written* part index,
 * unaffected by whichever `enabled_tracks` mutes that particular clip's
 * audio (e.g. "play selection" narrowing playback down to a few drag-selected
 * parts) — so a repeated/muted clip still resolves `note_id`s consistently.
 *
 * But the rendered SVG's `data-part-index` — what `usePlaybackCursor`'s DOM
 * lookups and `computeNoteSelectionTrimWindow`'s cell matching both key off
 * — is compacted by whichever parts are currently *hidden* via the part
 * visibility toggle: `apply_track_filter` physically removes hidden parts
 * before compiling, so hiding an earlier part shifts every later part's
 * index down (see `src/filters.rs`). That compaction is unrelated to (and,
 * for "play selection", a strict superset of) whatever narrower subset a
 * given clip mutes its audio down to.
 *
 * This remaps each timing's `source_part_index` from the true written index
 * into that same hidden-parts-compacted space, so it lines up with the SVG
 * regardless of which (if any) further subset of visible parts this
 * particular clip muted for playback. `visibleTracks` must be the part
 * visibility toggle's current state (never a playback-only mute override) —
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
