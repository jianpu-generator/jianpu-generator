/**
 * The distinct part abbreviations carried by a list of note/lyric spans or
 * selection runs, in first-seen order. Every span/run's `partAbbreviation` is
 * resolved in Rust (after hidden parts are filtered out — see
 * `note_spans::NoteSourceSpan::part_abbreviation`), so this never has to map
 * a `sourcePartIndex` back to a part name itself. An unnamed part (no
 * `partAbbreviation`) can't be targeted by name, so it's skipped.
 */
export function distinctPartAbbreviations(
  items: ReadonlyArray<{ partAbbreviation?: string }>,
): string[] {
  return Array.from(
    new Set(items.flatMap((item) => item.partAbbreviation ?? [])),
  )
}
