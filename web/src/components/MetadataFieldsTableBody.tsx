import type {
  MetadataDefaults,
  TextStyleDefaults,
} from '../utils/metadataDefaults'
import { metadataFieldHelp } from '../utils/metadataFieldHelp'
import type {
  MetadataFieldKey,
  ParsedMetadataFields,
  TextStyleComponent,
  TextStyleKind,
} from '../utils/metadataSource'
import {
  CheckboxFieldRow,
  NumberFieldRow,
  TextFieldRow,
  TextStyleRow,
} from './MetadataFieldRows'

export interface MetadataFieldsTableBodyProps {
  metadata: ParsedMetadataFields
  defaults: MetadataDefaults | null
  showHelp: (label: string, help: string) => void
  onTitleChange: (value: string) => void
  setText: (key: MetadataFieldKey) => (value: string) => void
  setNumber: (key: MetadataFieldKey) => (value: string) => void
  setYesNo: (
    key: MetadataFieldKey,
  ) => (e: React.ChangeEvent<HTMLInputElement>) => void
  setStyle: (
    kind: TextStyleKind,
  ) => (component: TextStyleComponent) => (value: string) => void
  numOrUndef: (n: number | null | undefined) => string | undefined
  titleFontSizeDefault: number | null
  subtitleFontSizeDefault: number | null
  authorFontSizeDefault: number | null
  partLegendFontSizeDefault: number | null
  lyricsFontSizeDefault: number | null
  effectiveLyricsFontSize: number | null
  effectiveNotesFontSize: number | null
  pageNumberFontSizeDefault: number | null
}

/** Overrides one kind's static `d?.<kind>` default with a live,
 * `row_height`- (or `lyrics_font_size`-) aware `font_size`, for the kinds
 * whose real default isn't a flat constant (see
 * `MetadataFieldsTableBodyProps`'s `*FontSizeDefault`/`effective*FontSize`
 * fields, resolved by `useFontSizeDefaults`/`EditMetadataModal`). Falls
 * back to the static snapshot (or `null`) when the live value isn't known
 * yet.
 */
function stylePlaceholder(
  base: TextStyleDefaults | undefined,
  liveFontSize?: number | null,
): TextStyleDefaults | null {
  if (!base) return null
  return liveFontSize == null ? base : { ...base, font_size: liveFontSize }
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
  setStyle,
  numOrUndef,
  titleFontSizeDefault,
  subtitleFontSizeDefault,
  authorFontSizeDefault,
  partLegendFontSizeDefault,
  lyricsFontSizeDefault,
  effectiveLyricsFontSize,
  effectiveNotesFontSize,
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
      <TextStyleRow
        label="Title Style"
        help={metadataFieldHelp.title}
        onShowHelp={showHelp}
        value={metadata.styles.title}
        placeholder={stylePlaceholder(d?.title, titleFontSizeDefault)}
        onChange={setStyle('title')}
      />
      <TextFieldRow
        label="Subtitle"
        help={metadataFieldHelp.subtitle}
        onShowHelp={showHelp}
        value={metadata.subtitle ?? ''}
        onChange={setText('subtitle')}
      />
      <TextStyleRow
        label="Subtitle Style"
        help={metadataFieldHelp.subtitle}
        onShowHelp={showHelp}
        value={metadata.styles.subtitle}
        placeholder={stylePlaceholder(d?.subtitle, subtitleFontSizeDefault)}
        onChange={setStyle('subtitle')}
      />
      <TextFieldRow
        label="Author"
        help={metadataFieldHelp.author}
        onShowHelp={showHelp}
        value={metadata.author ?? ''}
        onChange={setText('author')}
      />
      <TextStyleRow
        label="Author Style"
        help={metadataFieldHelp.author}
        onShowHelp={showHelp}
        value={metadata.styles.author}
        placeholder={stylePlaceholder(d?.author, authorFontSizeDefault)}
        onChange={setStyle('author')}
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
        label="Parts List Columns"
        help={metadataFieldHelp.parts_list_columns}
        onShowHelp={showHelp}
        value={metadata.parts_list_columns ?? ''}
        placeholder={numOrUndef(d?.parts_list_columns)}
        onChange={setNumber('parts_list_columns')}
      />
      <TextStyleRow
        label="Sequence Style"
        help={metadataFieldHelp.sequence}
        onShowHelp={showHelp}
        value={metadata.styles.sequence}
        placeholder={d?.sequence ?? null}
        onChange={setStyle('sequence')}
      />
      <TextStyleRow
        label="Part Legend Style"
        help={metadataFieldHelp.part_legend}
        onShowHelp={showHelp}
        value={metadata.styles.part_legend}
        placeholder={stylePlaceholder(
          d?.part_legend,
          partLegendFontSizeDefault,
        )}
        onChange={setStyle('part_legend')}
      />
      <TextStyleRow
        label="Measure Number Style"
        help={metadataFieldHelp.measure_number}
        onShowHelp={showHelp}
        value={metadata.styles.measure_number}
        placeholder={d?.measure_number ?? null}
        onChange={setStyle('measure_number')}
      />
      <TextStyleRow
        label="Section Label Style"
        help={metadataFieldHelp.section_label}
        onShowHelp={showHelp}
        value={metadata.styles.section_label}
        placeholder={d?.section_label ?? null}
        onChange={setStyle('section_label')}
      />
      <TextStyleRow
        label="Part Label Style"
        help={metadataFieldHelp.part_label}
        onShowHelp={showHelp}
        value={metadata.styles.part_label}
        placeholder={d?.part_label ?? null}
        onChange={setStyle('part_label')}
      />
      <NumberFieldRow
        label="Part Label Width"
        help={metadataFieldHelp.part_label_width_pt}
        onShowHelp={showHelp}
        value={metadata.part_label_width_pt ?? ''}
        placeholder={numOrUndef(d?.part_label_width_pt)}
        onChange={setNumber('part_label_width_pt')}
      />
      <TextStyleRow
        label="Page Number Style"
        help={metadataFieldHelp.page_number}
        onShowHelp={showHelp}
        value={metadata.styles.page_number}
        placeholder={stylePlaceholder(
          d?.page_number,
          pageNumberFontSizeDefault,
        )}
        onChange={setStyle('page_number')}
      />
      <TextStyleRow
        label="Lyrics Style"
        help={metadataFieldHelp.lyrics}
        onShowHelp={showHelp}
        value={metadata.styles.lyrics}
        placeholder={stylePlaceholder(d?.lyrics, lyricsFontSizeDefault)}
        onChange={setStyle('lyrics')}
      />
      <TextStyleRow
        label="Notes Style"
        help={metadataFieldHelp.notes}
        onShowHelp={showHelp}
        value={metadata.styles.notes}
        placeholder={stylePlaceholder(d?.notes, effectiveLyricsFontSize)}
        onChange={setStyle('notes')}
      />
      <TextStyleRow
        label="Chords Style"
        help={metadataFieldHelp.chords}
        onShowHelp={showHelp}
        value={metadata.styles.chords}
        placeholder={stylePlaceholder(d?.chords, effectiveLyricsFontSize)}
        onChange={setStyle('chords')}
      />
      <TextStyleRow
        label="Note Dash Style"
        help={metadataFieldHelp.note_dash}
        onShowHelp={showHelp}
        value={metadata.styles.note_dash}
        placeholder={stylePlaceholder(d?.note_dash, effectiveNotesFontSize)}
        onChange={setStyle('note_dash')}
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
