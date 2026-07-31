import type { MetadataKey } from './metadataSource'

/** Markdown help text shown in the field-help modal, describing precisely
 * which rendering/layout aspects each Edit Metadata field affects. */
export const metadataFieldHelp: Record<MetadataKey, string> = {
  title: `Rendered as the large heading at the top of the score's first page.`,

  subtitle: `Rendered as a smaller line beneath the title in the header.`,

  author: `Rendered in the header, below the title/subtitle.`,

  title_font_size: `Font size (points) of the title text.

Defaults to \`row_height × 1.5\` when unset.`,

  subtitle_font_size: `Font size (points) of the subtitle text.

Defaults to \`row_height × 0.8\` when unset.`,

  author_font_size: `Font size (points) of the author text.

Defaults to \`row_height × 0.6\` when unset.`,

  row_height: `Vertical spacing (points) of one part row.

Affects:
- Note heads and rests
- Octave and duration dots
- Tie/slur arc height
- Bar-line and multi-measure-rest thickness
- The default \`lyrics_font_size\` (unless that field is set explicitly), as \`row_height × 0.6\``,

  max_measures_per_system: `Maximum number of measures placed on one system (row) before wrapping to a new system line.`,

  note_number_width: `Offset (points) used to:
- Place an accidental (♯/♭) away from its note head
- Place a duration dot away from its note head
- Set the width of the underline drawn beneath eighth/sixteenth notes

**Has no visible effect** on a note with no accidental, no dot, and no underline.

Does **not** change the spacing between note columns — that comes from the available page width instead.`,

  part_label_width_pt: `Fixed width (points) of the part-label column at the start of each system.

This width is the same for every system in the score, regardless of how many measures or columns that system's music needs — so the part name lines up at the same horizontal position on every row.`,

  parts_list_columns: `Number of columns used to lay out the part-name legend shown in the header.`,

  part_legend_font_size: `Font size (points) of the part-name legend entries shown in the header.

Defaults to \`row_height × 0.6\` when unset.`,

  lyrics_font_size: `Font size (points) of lyric syllable text under notes.

Also affects how far a syllable is allowed to shift horizontally to avoid overlapping its neighbors.

Defaults to \`row_height × 0.6\` when unset.`,

  notes_font_size: `Font size (points) of note heads, rests, percussion hits, and tuplet brackets.

Also affects the width allotted to a note column, since these glyphs render as monospace characters.

Defaults to \`lyrics_font_size\` when unset.`,

  chords_font_size: `Font size (points) of chord symbol text.

Also affects the width allotted to a chord symbol's column.

Defaults to \`lyrics_font_size\` when unset.`,

  sequence_font_size: `Font size (points) of the \`# sequence\` summary line rendered near the top of the score.

Defaults to 12.`,

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
