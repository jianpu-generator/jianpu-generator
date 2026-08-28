import * as Dialog from '@radix-ui/react-dialog'
import { useEffect, useState } from 'react'
import type { MetadataDefaults } from '../utils/metadataDefaults'
import { loadMetadataDefaults } from '../utils/metadataDefaults'
import { metadataFieldHelp } from '../utils/metadataFieldHelp'
import type { MetadataKey, ParsedMetadataFields } from '../utils/metadataSource'
import { useFontSizeDefaults } from '../utils/useFontSizeDefaults'
import { FieldHelpModal } from './FieldHelpModal'
import {
  CheckboxFieldRow,
  NumberFieldRow,
  TextFieldRow,
} from './MetadataFieldRows'

export interface EditMetadataModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  metadata: ParsedMetadataFields
  onFieldChange: (key: MetadataKey, value: string | null) => void
  /** Element to confine the modal to (e.g. the editor pane), so it doesn't
   * cover the preview pane. Falls back to viewport-centered when null. */
  container?: HTMLElement | null
}

const thStyle: React.CSSProperties = {
  padding: '6px 10px',
  textAlign: 'left',
  fontWeight: 600,
  fontSize: '12px',
  color: '#444',
  borderBottom: '2px solid #ddd',
  background: '#f5f5f5',
}

export function EditMetadataModal({
  open,
  onOpenChange,
  metadata,
  onFieldChange,
  container,
}: EditMetadataModalProps) {
  const [defaults, setDefaults] = useState<MetadataDefaults | null>(null)
  const [helpContent, setHelpContent] = useState<{
    label: string
    help: string
  } | null>(null)
  const showHelp = (label: string, help: string) =>
    setHelpContent({ label, help })

  useEffect(() => {
    loadMetadataDefaults().then(setDefaults)
  }, [])

  const effectiveRowHeight = metadata.row_height ?? defaults?.row_height ?? null
  const {
    lyricsFontSizeDefault,
    titleFontSizeDefault,
    subtitleFontSizeDefault,
    authorFontSizeDefault,
    partLegendFontSizeDefault,
    pageNumberFontSizeDefault,
  } = useFontSizeDefaults(effectiveRowHeight)

  const d = defaults

  // notes_font_size/chords_font_size default to the *effective* lyrics font
  // size — either the explicit override or its own row_height-derived default.
  const effectiveLyricsFontSize =
    metadata.lyrics_font_size ?? lyricsFontSizeDefault

  const setText =
    (key: MetadataKey) => (e: React.ChangeEvent<HTMLInputElement>) =>
      onFieldChange(key, e.target.value === '' ? null : e.target.value)

  const setNumber =
    (key: MetadataKey) => (e: React.ChangeEvent<HTMLInputElement>) =>
      onFieldChange(key, e.target.value === '' ? null : e.target.value)

  const setYesNo =
    (key: MetadataKey) => (e: React.ChangeEvent<HTMLInputElement>) =>
      onFieldChange(key, e.target.checked ? 'yes' : 'no')

  const numOrUndef = (n: number | null | undefined): string | undefined =>
    n != null ? String(n) : undefined

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange} modal={false}>
      <Dialog.Portal container={container ?? undefined}>
        <Dialog.Overlay
          style={{
            position: container ? 'absolute' : 'fixed',
            inset: 0,
            background: 'rgba(0,0,0,0.35)',
            zIndex: 1000,
          }}
        />
        <Dialog.Content
          data-testid="edit-metadata-modal"
          style={{
            position: container ? 'absolute' : 'fixed',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            background: '#fff',
            border: '1px solid #ddd',
            borderRadius: '6px',
            boxShadow: '0 8px 32px rgba(0,0,0,0.16)',
            zIndex: 1001,
            minWidth: container ? undefined : '420px',
            width: container ? '90%' : undefined,
            maxWidth: container ? undefined : '90vw',
            maxHeight: container ? '90%' : '80vh',
            display: 'flex',
            flexDirection: 'column',
            fontFamily: 'var(--mono, monospace)',
          }}
        >
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: '12px 16px',
              borderBottom: '1px solid #eee',
            }}
          >
            <Dialog.Title
              style={{ margin: 0, fontSize: '14px', fontWeight: 600 }}
            >
              Edit Metadata
            </Dialog.Title>
            <Dialog.Close
              style={{
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                fontSize: '16px',
                color: '#666',
                lineHeight: 1,
                padding: '2px 4px',
              }}
            >
              ×
            </Dialog.Close>
          </div>
          <div style={{ overflowY: 'auto', flex: 1 }}>
            <table
              style={{
                width: '100%',
                borderCollapse: 'collapse',
                tableLayout: 'fixed',
              }}
            >
              <colgroup>
                <col style={{ width: '40%' }} />
                <col style={{ width: '60%' }} />
              </colgroup>
              <thead>
                <tr>
                  <th style={thStyle}>Field</th>
                  <th style={thStyle}>Value</th>
                </tr>
              </thead>
              <tbody>
                <TextFieldRow
                  label="Title *"
                  help={metadataFieldHelp.title}
                  onShowHelp={showHelp}
                  value={metadata.title}
                  onChange={(e) => onFieldChange('title', e.target.value)}
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
                  checked={
                    metadata.hide_resting_parts ?? d?.hide_resting_parts ?? true
                  }
                  onChange={setYesNo('hide_resting_parts')}
                />
                <CheckboxFieldRow
                  label="Hide System Dividers"
                  help={metadataFieldHelp.hide_system_dividers}
                  onShowHelp={showHelp}
                  checked={
                    metadata.hide_system_dividers ??
                    d?.hide_system_dividers ??
                    false
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
            </table>
          </div>
          <FieldHelpModal
            content={helpContent}
            onOpenChange={(open) => !open && setHelpContent(null)}
          />
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
