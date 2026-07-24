import type { NoteTimingOut } from 'jianpu-wasm'

export interface ActiveNote {
  sourcePartIndex: number
  noteId: number
}

/**
 * Groups `noteTimings` by `source_part_index`, sorting each part's list
 * ascending by `start_s` (already the order `note_timings_seconds` produces
 * per part, but sorted defensively so `resolveActiveNotes`'s binary search
 * doesn't depend on that).
 */
export function groupNoteTimingsByPart(
  noteTimings: NoteTimingOut[],
): Map<number, NoteTimingOut[]> {
  const byPart = new Map<number, NoteTimingOut[]>()
  for (const timing of noteTimings) {
    const list = byPart.get(timing.source_part_index)
    if (list) {
      list.push(timing)
    } else {
      byPart.set(timing.source_part_index, [timing])
    }
  }
  for (const list of byPart.values()) {
    list.sort((a, b) => a.start_s - b.start_s)
  }
  return byPart
}

/**
 * Binary search for the last timing whose `start_s <= t` in `timings`
 * (must be sorted ascending by `start_s`). Returns `-1` when `t` is before
 * every timing's start.
 */
function findLastTimingStartingAtOrBefore(
  timings: NoteTimingOut[],
  t: number,
): number {
  let lo = 0
  let hi = timings.length - 1
  let result = -1
  while (lo <= hi) {
    const mid = (lo + hi) >> 1
    if (timings[mid].start_s <= t) {
      result = mid
      lo = mid + 1
    } else {
      hi = mid - 1
    }
  }
  return result
}

/**
 * The active `(source_part_index, note_id)` per part at time `t`, given
 * `timingsByPart` (see `groupNoteTimingsByPart`). A part with no note
 * sounding at `t` — before its first note starts, past its last note's end,
 * or in a gap between timings — is omitted from the result rather than
 * falling back to a neighboring note, since there is nothing correct to
 * highlight there.
 */
export function resolveActiveNotes(
  t: number,
  timingsByPart: Map<number, NoteTimingOut[]>,
): ActiveNote[] {
  const active: ActiveNote[] = []
  for (const [sourcePartIndex, timings] of timingsByPart) {
    const index = findLastTimingStartingAtOrBefore(timings, t)
    if (index === -1) continue
    const timing = timings[index]
    if (t >= timing.end_s) continue
    active.push({ sourcePartIndex, noteId: timing.note_id })
  }
  return active
}
