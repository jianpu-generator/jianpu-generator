import type {
  MetadataEdit,
  MetadataFields,
  MetadataFlagKey,
  MetadataNumberKey,
  MetadataTextKey,
} from '../jianpuWasm'
import { metadataFieldHelp } from '../utils/metadataFieldHelp'
import {
  CheckboxFieldRow,
  NumberFieldRow,
  TextFieldRow,
} from './MetadataFieldRows'

export interface MetadataFieldsTableBodyProps {
  metadata: MetadataFields
  showHelp: (label: string, help: string) => void
  onFieldChange: (edit: MetadataEdit) => void
}

/** An input's text as an optional value: empty clears the field. */
export function optionalText(value: string): string | undefined {
  return value === '' ? undefined : value
}

/** A number input's text as an optional value: empty (or unparseable)
 * clears the field. */
export function optionalNumber(value: string): number | undefined {
  const parsed = Number.parseInt(value, 10)
  return Number.isNaN(parsed) ? undefined : parsed
}

/** The `<tbody>` of `EditMetadataModal`'s plain (non-text-style) field
 * table — split out to keep that file under the repo's max-file-lines cap.
 * Text-style kinds (Title Style, Subtitle Style, ...) live in the separate
 * `MetadataStylesTable`, not here. */
export function MetadataFieldsTableBody({
  metadata,
  showHelp,
  onFieldChange,
}: MetadataFieldsTableBodyProps) {
  const d = metadata.defaults
  const setText = (key: MetadataTextKey) => (value: string) =>
    onFieldChange({ tag: 'text', val: { key, value: optionalText(value) } })
  const setNumber = (key: MetadataNumberKey) => (value: string) =>
    onFieldChange({ tag: 'number', val: { key, value: optionalNumber(value) } })
  const setFlag =
    (key: MetadataFlagKey) => (e: React.ChangeEvent<HTMLInputElement>) =>
      onFieldChange({ tag: 'flag', val: { key, value: e.target.checked } })

  return (
    <tbody>
      <TextFieldRow
        label="Title *"
        help={metadataFieldHelp.title}
        onShowHelp={showHelp}
        value={metadata.title ?? ''}
        onChange={setText('title')}
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
        help={metadataFieldHelp['row-height']}
        onShowHelp={showHelp}
        value={metadata.rowHeight ?? ''}
        placeholder={String(d.rowHeight)}
        onChange={setNumber('row-height')}
      />
      <NumberFieldRow
        label="Max Measures Per System"
        help={metadataFieldHelp['max-measures-per-system']}
        onShowHelp={showHelp}
        value={metadata.maxMeasuresPerSystem ?? ''}
        placeholder={String(d.maxMeasuresPerSystem)}
        onChange={setNumber('max-measures-per-system')}
      />
      <NumberFieldRow
        label="Note Number Width"
        help={metadataFieldHelp['note-number-width']}
        onShowHelp={showHelp}
        value={metadata.noteNumberWidth ?? ''}
        placeholder={String(d.noteNumberWidth)}
        onChange={setNumber('note-number-width')}
      />
      <NumberFieldRow
        label="Parts List Columns"
        help={metadataFieldHelp['parts-list-columns']}
        onShowHelp={showHelp}
        value={metadata.partsListColumns ?? ''}
        placeholder={String(d.partsListColumns)}
        onChange={setNumber('parts-list-columns')}
      />
      <NumberFieldRow
        label="Part Label Width"
        help={metadataFieldHelp['part-label-width-pt']}
        onShowHelp={showHelp}
        value={metadata.partLabelWidthPt ?? ''}
        placeholder={String(d.partLabelWidthPt)}
        onChange={setNumber('part-label-width-pt')}
      />
      <CheckboxFieldRow
        label="Merge Duplicate Measures Across Parts"
        help={metadataFieldHelp['merge-duplicate-measures-across-parts']}
        onShowHelp={showHelp}
        checked={
          metadata.mergeDuplicateMeasuresAcrossParts ??
          d.mergeDuplicateMeasuresAcrossParts
        }
        onChange={setFlag('merge-duplicate-measures-across-parts')}
      />
      <CheckboxFieldRow
        label="Hide Resting Parts"
        help={metadataFieldHelp['hide-resting-parts']}
        onShowHelp={showHelp}
        checked={metadata.hideRestingParts ?? d.hideRestingParts}
        onChange={setFlag('hide-resting-parts')}
      />
      <CheckboxFieldRow
        label="Hide System Dividers"
        help={metadataFieldHelp['hide-system-dividers']}
        onShowHelp={showHelp}
        checked={metadata.hideSystemDividers ?? d.hideSystemDividers}
        onChange={setFlag('hide-system-dividers')}
      />
      <TextFieldRow
        label="Directive Row Offset (x y)"
        help={metadataFieldHelp['directive-row-offset']}
        onShowHelp={showHelp}
        value={metadata.directiveRowOffset ?? ''}
        placeholder={d.directiveRowOffset}
        onChange={(value) =>
          onFieldChange({
            tag: 'directive-row-offset',
            val: optionalText(value),
          })
        }
      />
    </tbody>
  )
}
