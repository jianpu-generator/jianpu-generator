import type { MetadataDefaults } from '../utils/metadataDefaults'
import { metadataFieldHelp } from '../utils/metadataFieldHelp'
import type {
  MetadataFieldKey,
  ParsedMetadataFields,
} from '../utils/metadataSource'
import {
  CheckboxFieldRow,
  NumberFieldRow,
  TextFieldRow,
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
  numOrUndef: (n: number | null | undefined) => string | undefined
}

/** The `<tbody>` of `EditMetadataModal`'s plain (non-text-style) field
 * table — split out to keep that file under the repo's max-file-lines cap.
 * Text-style kinds (Title Style, Subtitle Style, ...) live in the separate
 * `MetadataStylesTable`, not here. */
export function MetadataFieldsTableBody({
  metadata,
  defaults: d,
  showHelp,
  onTitleChange,
  setText,
  setNumber,
  setYesNo,
  numOrUndef,
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
      <TextFieldRow
        label="Subtitle"
        help={metadataFieldHelp.subtitle}
        onShowHelp={showHelp}
        value={metadata.subtitle ?? ''}
        onChange={setText('subtitle')}
      />
      <TextFieldRow
        label="Author"
        help={metadataFieldHelp.author}
        onShowHelp={showHelp}
        value={metadata.author ?? ''}
        onChange={setText('author')}
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
      <NumberFieldRow
        label="Part Label Width"
        help={metadataFieldHelp.part_label_width_pt}
        onShowHelp={showHelp}
        value={metadata.part_label_width_pt ?? ''}
        placeholder={numOrUndef(d?.part_label_width_pt)}
        onChange={setNumber('part_label_width_pt')}
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
