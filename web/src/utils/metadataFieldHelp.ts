import type { TextStyleKind } from './metadataSource'

/** Markdown help text shown in the field-help modal. Keyed by the text
 * content fields (`title`/`subtitle`/`author`), the 13 `TextStyleKind`s
 * (shared by both the content row and its style row, for the three that
 * have both), the scalar fields, and the checkbox fields — describing
 * precisely which rendering/layout aspects each Edit Metadata field
 * affects. */
export const metadataFieldHelp: Record<
  | 'title'
  | 'subtitle'
  | 'author'
  | TextStyleKind
  | 'row_height'
  | 'max_measures_per_system'
  | 'note_number_width'
  | 'parts_list_columns'
  | 'merge_duplicate_measures_across_parts'
  | 'hide_resting_parts'
  | 'hide_system_dividers'
  | 'directive_row_offset',
  string
> = {
  title: `Rendered as the large heading at the top of the score's first page.

Its style row's **Font Size** defaults to \`row_height × 1.5\`; **Width** reserves a minimum box width for the rendered title (default: 0, no minimum).`,

  subtitle: `Rendered as a smaller line beneath the title in the header.

Its style row's **Font Size** defaults to \`row_height × 0.8\`.`,

  author: `Rendered in the header, below the title/subtitle.

Its style row's **Font Size** defaults to \`row_height × 0.6\`.`,

  sequence: `Style of the \`# sequence\` summary line rendered near the top of the score.

**Font Size** defaults to 12.`,

  part_legend: `Style of the part-name legend entries shown in the header.

**Font Size** defaults to \`row_height × 0.6\`.`,

  measure_number: `Style of each measure's bar number.

**Font Size** defaults to 10.`,

  section_label: `Style of an inline section label (the \`label="..."\` on a measure's directive line).

**Font Size** defaults to 12.`,

  part_label: `Style of a part's row label (e.g. "Soprano"), shown at the start of each system row.

**Font Size** defaults to 12. **Width** is the fixed width (points) of the part-label column at the start of each system, shared by every system in the score regardless of how many measures/columns that system's music needs (default: 40).`,

  page_number: `Style of the page number shown in the footer.

**Font Size** defaults to \`row_height × 0.6\`. **V. Padding** pushes the page number upward from the page's bottom edge, without moving anything else.`,

  lyrics: `Style of lyric syllable text under notes.

**Font Size** defaults to \`row_height × 0.6\` and also affects how far a syllable is allowed to shift horizontally to avoid overlapping its neighbors. **H. Padding** defaults to 4 (widens spacing between syllables). **V. Padding** is extra padding around a lyric syllable's hover/click-target box, on top of the lyric font's own measured height (default: 12).`,

  notes: `Style of note heads, rests, percussion hits, and tuplet brackets.

**Font Size** defaults to \`lyrics.font_size\` and also affects the width allotted to a note column, since these glyphs render as monospace characters. **H. Padding** (default: 4) is also used for the multi-measure-rest bar's end insets and the tie/slur/underline/tuplet-bracket markings, which all key off a note column. **V. Padding** adds vertical space around the note-head row.`,

  chords: `Style of chord symbol text.

**Font Size** defaults to \`lyrics.font_size\` and also affects the width allotted to a chord symbol's column. **H. Padding** defaults to 4.`,

  note_dash: `Style of a note dash (the sustain-beat \`-\` extension).

**Font Size** defaults to \`notes.font_size\` and scales the rendered dash's width. **H. Padding** defaults to 4.`,

  row_height: `Vertical spacing (points) of one part row.

Affects:
- Note heads and rests
- Octave and duration dots
- Tie/slur arc height
- Bar-line and multi-measure-rest thickness
- The default \`lyrics\` style's font size (unless set explicitly), as \`row_height × 0.6\``,

  max_measures_per_system: `Maximum number of measures placed on one system (row) before wrapping to a new system line.`,

  note_number_width: `Offset (points) used to:
- Place an accidental (♯/♭) away from its note head
- Place a duration dot away from its note head
- Set the width of the underline drawn beneath eighth/sixteenth notes

**Has no visible effect** on a note with no accidental, no dot, and no underline.

Does **not** change the spacing between note columns — that comes from the available page width instead.`,

  parts_list_columns: `Number of columns used to lay out the part-name legend shown in the header.`,

  merge_duplicate_measures_across_parts: `When on, measures with identical content across different parts are drawn as a single merged row instead of one row per part.

Can be overridden from a specific measure onward with a
\`merge_duplicate_measures_across_parts=yes\`/\`no\` directive line.`,

  hide_resting_parts: `When on, a part that is entirely rests in a measure is omitted from that measure's system whenever at least one other part has real content.

Can be overridden from a specific measure onward with a
\`hide_resting_parts=yes\`/\`no\` directive line.`,

  hide_system_dividers: `When on, the horizontal divider line normally drawn between consecutive systems (rows of measures) is omitted.`,

  directive_row_offset: `Translation (points, \`"x y"\`) applied to every rendered directive row — bar number, section label, key, bpm, and time signature.

Moves that row's text without affecting the layout or spacing of anything else on the page.

Not applied to the \`# sequence\` summary header line.`,
}
