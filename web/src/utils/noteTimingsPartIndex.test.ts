import type { NoteTimingOut, PartOut } from 'jianpu-wasm'
import { describe, expect, it } from 'vitest'
import { remapNoteTimingsToVisiblePartIndex } from './noteTimingsPartIndex'

function part(abbreviation: string): PartOut {
  return { abbreviation, display_name: abbreviation, has_lyrics: false }
}

function timing(sourcePartIndex: number, noteId: number): NoteTimingOut {
  return {
    source_part_index: sourcePartIndex,
    note_id: noteId,
    start_s: 0,
    end_s: 1,
  }
}

describe('remapNoteTimingsToVisiblePartIndex', () => {
  it('returns timings unchanged when no part is hidden', () => {
    const parts = [part('M'), part('H'), part('B')]
    const timings = [timing(0, 0), timing(1, 0), timing(2, 0)]
    expect(remapNoteTimingsToVisiblePartIndex(timings, parts, undefined)).toBe(
      timings,
    )
  })

  it('shifts a later part down when an earlier part is hidden', () => {
    // Melody (written 0), Harmony (written 1, hidden), Bass (written 2) ->
    // Bass compacts to rendered index 1, matching the SVG's data-part-index.
    const parts = [part('M'), part('H'), part('B')]
    const timings = [timing(0, 0), timing(2, 0)]
    const remapped = remapNoteTimingsToVisiblePartIndex(timings, parts, [
      'M',
      'B',
    ])
    expect(remapped).toEqual([timing(0, 0), timing(1, 0)])
  })

  it('drops timings belonging to a hidden part', () => {
    const parts = [part('M'), part('H'), part('B')]
    const timings = [timing(0, 0), timing(1, 0), timing(2, 0)]
    const remapped = remapNoteTimingsToVisiblePartIndex(timings, parts, [
      'M',
      'B',
    ])
    expect(remapped).toEqual([timing(0, 0), timing(1, 0)])
  })

  it('keeps the compacted index space independent of a narrower playback-only mute', () => {
    // Harmony hidden (visibleTracks = M, B), and this particular clip further
    // mutes down to just Bass for "play selection" — Bass's *written* index
    // is still 2, and must resolve to compacted index 1 (its position among
    // the *visible* parts), not 0 (its position among the *muted* subset).
    const parts = [part('M'), part('H'), part('B')]
    const timings = [timing(2, 0)]
    const remapped = remapNoteTimingsToVisiblePartIndex(timings, parts, [
      'M',
      'B',
    ])
    expect(remapped).toEqual([timing(1, 0)])
  })
})
