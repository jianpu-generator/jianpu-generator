import type { MetadataDefaults } from '../utils/metadataDefaults'
import { metadataFieldHelp } from '../utils/metadataFieldHelp'
import type { MetadataKey, ParsedMetadataFields } from '../utils/metadataSource'
import {
  CheckboxFieldRow,
  NumberFieldRow,
  TextFieldRow,
} from './MetadataFieldRows'

export interface MetadataFieldsTableBodyProps {
  metadata: ParsedMetadataFields
  defaults: MetadataDefaults | null
  showHelp: (label: string, help: string) => void
  onTitleChange: (e: React.ChangeEvent<HTMLInputElement>) => void
  setText: (
    key: MetadataKey,
  ) => (e: React.ChangeEvent<HTMLInputElement>) => void
  setNumber: (
    key: MetadataKey,
  ) => (e: React.ChangeEvent<HTMLInputElement>) => void
  setYesNo: (
    key: MetadataKey,
  ) => (e: React.ChangeEvent<HTMLInputElement>) => void
  numOrUndef: (n: number | null | undefined) => string | undefined
  titleFontSizeDefault: number | null
  subtitleFontSizeDefault: number | null
  authorFontSizeDefault: number | null
  partLegendFontSizeDefault: number | null
  lyricsFontSizeDefault: number | null
  effectiveLyricsFontSize: number | null
  pageNumberFontSizeDefault: number | null
}

/** The `<tbody>` of `EditMetadataModal`'s field table — split out to keep
 * that file under the repo's max-file-lines cap. */
export function MetadataFieldsTableBody({
  metadata,
  defaults: d,
  showHelp,
  onTitleChange,
  setText,
  setNumber,
  setYesNo,
  numOrUndef,
  titleFontSizeDefault,
  subtitleFontSizeDefault,
  authorFontSizeDefault,
  partLegendFontSizeDefault,
  lyricsFontSizeDefault,
  effectiveLyricsFontSize,
  pageNumberFontSizeDefault,
}: MetadataFieldsTableBodyProps) {
  return (
    <tbody>
      <TextFieldRow
        label="Title *"
        help={metadataFieldHelp.title}
        onShowHelp={showHelp}
        value={metadata.title}
        onChange={onTitleChange}
      />
      <NumberFieldRow
        label="Title Font Size"
        help={metadataFieldHelp.title_font_size}
        onShowHelp={showHelp}
        value={metadata.title_font_size ?? ''}
        placeholder={numOrUndef(titleFontSizeDefault)}
        onChange={setNumber('title_font_size')}
      />
      <TextFieldRow
        label="Subtitle"
        help={metadataFieldHelp.subtitle}
        onShowHelp={showHelp}
        value={metadata.subtitle ?? ''}
        onChange={setText('subtitle')}
      />
      <NumberFieldRow
        label="Subtitle Font Size"
        help={metadataFieldHelp.subtitle_font_size}
        onShowHelp={showHelp}
        value={metadata.subtitle_font_size ?? ''}
        placeholder={numOrUndef(subtitleFontSizeDefault)}
        onChange={setNumber('subtitle_font_size')}
      />
      <TextFieldRow
        label="Author"
        help={metadataFieldHelp.author}
        onShowHelp={showHelp}
        value={metadata.author ?? ''}
        onChange={setText('author')}
      />
      <NumberFieldRow
        label="Author Font Size"
        help={metadataFieldHelp.author_font_size}
        onShowHelp={showHelp}
        value={metadata.author_font_size ?? ''}
        placeholder={numOrUndef(authorFontSizeDefault)}
        onChange={setNumber('author_font_size')}
      />
      <NumberFieldRow
        label="Row Height"
        help={metadataFieldHelp.row_height}
        onShowHelp={showHelp}
        value={metadata.row_height ?? ''}
        placeholder={numOrUndef(d?.row_height)}
        onChange={setNumber('row_height')}
      />
      <NumberFieldRow
        label="Max Measures Per System"
        help={metadataFieldHelp.max_measures_per_system}
        onShowHelp={showHelp}
        value={metadata.max_measures_per_system ?? ''}
        placeholder={numOrUndef(d?.max_measures_per_system)}
        onChange={setNumber('max_measures_per_system')}
      />
      <NumberFieldRow
        label="Note Number Width"
        help={metadataFieldHelp.note_number_width}
        onShowHelp={showHelp}
        value={metadata.note_number_width ?? ''}
        placeholder={numOrUndef(d?.note_number_width)}
        onChange={setNumber('note_number_width')}
      />
      <NumberFieldRow
        label="Part Label Width (pt)"
        help={metadataFieldHelp.part_label_width_pt}
        onShowHelp={showHelp}
        value={metadata.part_label_width_pt ?? ''}
        placeholder={numOrUndef(d?.part_label_width_pt)}
        onChange={setNumber('part_label_width_pt')}
      />
      <NumberFieldRow
        label="Parts List Columns"
        help={metadataFieldHelp.parts_list_columns}
        onShowHelp={showHelp}
        value={metadata.parts_list_columns ?? ''}
        placeholder={numOrUndef(d?.parts_list_columns)}
        onChange={setNumber('parts_list_columns')}
      />
      <NumberFieldRow
        label="Part Legend Font Size"
        help={metadataFieldHelp.part_legend_font_size}
        onShowHelp={showHelp}
        value={metadata.part_legend_font_size ?? ''}
        placeholder={numOrUndef(partLegendFontSizeDefault)}
        onChange={setNumber('part_legend_font_size')}
      />
      <NumberFieldRow
        label="Lyrics Font Size"
        help={metadataFieldHelp.lyrics_font_size}
        onShowHelp={showHelp}
        value={metadata.lyrics_font_size ?? ''}
        placeholder={numOrUndef(lyricsFontSizeDefault)}
        onChange={setNumber('lyrics_font_size')}
      />
      <NumberFieldRow
        label="Notes Font Size"
        help={metadataFieldHelp.notes_font_size}
        onShowHelp={showHelp}
        value={metadata.notes_font_size ?? ''}
        placeholder={numOrUndef(effectiveLyricsFontSize)}
        onChange={setNumber('notes_font_size')}
      />
      <NumberFieldRow
        label="Chords Font Size"
        help={metadataFieldHelp.chords_font_size}
        onShowHelp={showHelp}
        value={metadata.chords_font_size ?? ''}
        placeholder={numOrUndef(effectiveLyricsFontSize)}
        onChange={setNumber('chords_font_size')}
      />
      <NumberFieldRow
        label="Sequence Font Size"
        help={metadataFieldHelp.sequence_font_size}
        onShowHelp={showHelp}
        value={metadata.sequence_font_size ?? ''}
        placeholder={numOrUndef(d?.sequence_font_size)}
        onChange={setNumber('sequence_font_size')}
      />
      <NumberFieldRow
        label="Measure Number Font Size"
        help={metadataFieldHelp.measure_number_font_size}
        onShowHelp={showHelp}
        value={metadata.measure_number_font_size ?? ''}
        placeholder={numOrUndef(d?.measure_number_font_size)}
        onChange={setNumber('measure_number_font_size')}
      />
      <NumberFieldRow
        label="Section Label Font Size"
        help={metadataFieldHelp.section_label_font_size}
        onShowHelp={showHelp}
        value={metadata.section_label_font_size ?? ''}
        placeholder={numOrUndef(d?.section_label_font_size)}
        onChange={setNumber('section_label_font_size')}
      />
      <NumberFieldRow
        label="Part Label Font Size"
        help={metadataFieldHelp.part_label_font_size}
        onShowHelp={showHelp}
        value={metadata.part_label_font_size ?? ''}
        placeholder={numOrUndef(d?.part_label_font_size)}
        onChange={setNumber('part_label_font_size')}
      />
      <NumberFieldRow
        label="Page Number Font Size"
        help={metadataFieldHelp.page_number_font_size}
        onShowHelp={showHelp}
        value={metadata.page_number_font_size ?? ''}
        placeholder={numOrUndef(pageNumberFontSizeDefault)}
        onChange={setNumber('page_number_font_size')}
      />
      <NumberFieldRow
        label="Lyric Click-Target Padding"
        help={metadataFieldHelp.lyric_click_target_padding_pt}
        onShowHelp={showHelp}
        value={metadata.lyric_click_target_padding_pt ?? ''}
        placeholder={numOrUndef(d?.lyric_click_target_padding_pt)}
        onChange={setNumber('lyric_click_target_padding_pt')}
      />
      <NumberFieldRow
        label="Notes Horizontal Padding"
        help={metadataFieldHelp.notes_horizontal_padding_pt}
        onShowHelp={showHelp}
        value={metadata.notes_horizontal_padding_pt ?? ''}
        placeholder={numOrUndef(d?.notes_horizontal_padding_pt)}
        onChange={setNumber('notes_horizontal_padding_pt')}
      />
      <NumberFieldRow
        label="Chords Horizontal Padding"
        help={metadataFieldHelp.chords_horizontal_padding_pt}
        onShowHelp={showHelp}
        value={metadata.chords_horizontal_padding_pt ?? ''}
        placeholder={numOrUndef(d?.chords_horizontal_padding_pt)}
        onChange={setNumber('chords_horizontal_padding_pt')}
      />
      <NumberFieldRow
        label="Lyrics Horizontal Padding"
        help={metadataFieldHelp.lyrics_horizontal_padding_pt}
        onShowHelp={showHelp}
        value={metadata.lyrics_horizontal_padding_pt ?? ''}
        placeholder={numOrUndef(d?.lyrics_horizontal_padding_pt)}
        onChange={setNumber('lyrics_horizontal_padding_pt')}
      />
      <NumberFieldRow
        label="Note Dash Horizontal Padding"
        help={metadataFieldHelp.note_dash_horizontal_padding_pt}
        onShowHelp={showHelp}
        value={metadata.note_dash_horizontal_padding_pt ?? ''}
        placeholder={numOrUndef(d?.note_dash_horizontal_padding_pt)}
        onChange={setNumber('note_dash_horizontal_padding_pt')}
      />
      <CheckboxFieldRow
        label="Merge Duplicate Measures Across Parts"
        help={metadataFieldHelp.merge_duplicate_measures_across_parts}
        onShowHelp={showHelp}
        checked={
          metadata.merge_duplicate_measures_across_parts ??
          d?.merge_duplicate_measures_across_parts ??
          true
        }
        onChange={setYesNo('merge_duplicate_measures_across_parts')}
      />
      <CheckboxFieldRow
        label="Hide Resting Parts"
        help={metadataFieldHelp.hide_resting_parts}
        onShowHelp={showHelp}
        checked={metadata.hide_resting_parts ?? d?.hide_resting_parts ?? true}
        onChange={setYesNo('hide_resting_parts')}
      />
      <CheckboxFieldRow
        label="Hide System Dividers"
        help={metadataFieldHelp.hide_system_dividers}
        onShowHelp={showHelp}
        checked={
          metadata.hide_system_dividers ?? d?.hide_system_dividers ?? false
        }
        onChange={setYesNo('hide_system_dividers')}
      />
      <TextFieldRow
        label="Directive Row Offset (x y)"
        help={metadataFieldHelp.directive_row_offset}
        onShowHelp={showHelp}
        value={metadata.directive_row_offset ?? ''}
        placeholder={
          d
            ? `${d.directive_row_offset_x} ${d.directive_row_offset_y}`
            : undefined
        }
        onChange={setText('directive_row_offset')}
      />
    </tbody>
  )
}
