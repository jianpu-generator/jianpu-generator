import * as Dialog from '@radix-ui/react-dialog'
import { useEffect, useState } from 'react'
import type {
  MetadataDefaults,
  TextStyleDefaults,
} from '../utils/metadataDefaults'
import { loadMetadataDefaults } from '../utils/metadataDefaults'
import { metadataFieldHelp } from '../utils/metadataFieldHelp'
import type {
  MetadataFieldKey,
  ParsedMetadataFields,
  TextStyleComponent,
  TextStyleKind,
} from '../utils/metadataSource'
import { useFontSizeDefaults } from '../utils/useFontSizeDefaults'
import { FieldHelpModal } from './FieldHelpModal'
import { MetadataFieldsTableBody } from './MetadataFieldsTableBody'
import type { StyleRowSpec } from './MetadataStylesTable'
import { MetadataStylesTable } from './MetadataStylesTable'

export interface EditMetadataModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  metadata: ParsedMetadataFields
  onFieldChange: (key: MetadataFieldKey, value: string | null) => void
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

  // notes/chords styles' font_size default to the *effective* lyrics font
  // size — either the explicit override or its own row_height-derived
  // default; note_dash's font_size then defaults to that effective notes
  // font size, one level further down the cascade (see `syntax.md`'s
  // "Text styles" defaults table).
  const effectiveLyricsFontSize =
    metadata.styles.lyrics.font_size ?? lyricsFontSizeDefault
  const effectiveNotesFontSize =
    metadata.styles.notes.font_size ?? effectiveLyricsFontSize

  const setText = (key: MetadataFieldKey) => (value: string) =>
    onFieldChange(key, value === '' ? null : value)

  const setNumber = (key: MetadataFieldKey) => (value: string) =>
    onFieldChange(key, value === '' ? null : value)

  const setYesNo =
    (key: MetadataFieldKey) => (e: React.ChangeEvent<HTMLInputElement>) =>
      onFieldChange(key, e.target.checked ? 'yes' : 'no')

  const setStyle =
    (kind: TextStyleKind) =>
    (component: TextStyleComponent) =>
    (value: string) =>
      onFieldChange(`${kind}.${component}`, value === '' ? null : value)

  const numOrUndef = (n: number | null | undefined): string | undefined =>
    n != null ? String(n) : undefined

  // Overrides one kind's static `d?.<kind>` default with a live,
  // `row_height`- (or `lyrics_font_size`-) aware `font_size`, for the kinds
  // whose real default isn't a flat constant (see `useFontSizeDefaults`
  // above). Falls back to the static snapshot (or `null`) when the live
  // value isn't known yet.
  const stylePlaceholder = (
    base: TextStyleDefaults | undefined,
    liveFontSize?: number | null,
  ): TextStyleDefaults | null => {
    if (!base) return null
    return liveFontSize == null ? base : { ...base, font_size: liveFontSize }
  }

  const styleRows: StyleRowSpec[] = [
    {
      kind: 'title',
      label: 'Title Style',
      help: metadataFieldHelp.title,
      value: metadata.styles.title,
      placeholder: stylePlaceholder(d?.title, titleFontSizeDefault),
    },
    {
      kind: 'subtitle',
      label: 'Subtitle Style',
      help: metadataFieldHelp.subtitle,
      value: metadata.styles.subtitle,
      placeholder: stylePlaceholder(d?.subtitle, subtitleFontSizeDefault),
    },
    {
      kind: 'author',
      label: 'Author Style',
      help: metadataFieldHelp.author,
      value: metadata.styles.author,
      placeholder: stylePlaceholder(d?.author, authorFontSizeDefault),
    },
    {
      kind: 'sequence',
      label: 'Sequence Style',
      help: metadataFieldHelp.sequence,
      value: metadata.styles.sequence,
      placeholder: d?.sequence ?? null,
    },
    {
      kind: 'part_legend',
      label: 'Part Legend Style',
      help: metadataFieldHelp.part_legend,
      value: metadata.styles.part_legend,
      placeholder: stylePlaceholder(d?.part_legend, partLegendFontSizeDefault),
    },
    {
      kind: 'measure_number',
      label: 'Measure Number Style',
      help: metadataFieldHelp.measure_number,
      value: metadata.styles.measure_number,
      placeholder: d?.measure_number ?? null,
    },
    {
      kind: 'section_label',
      label: 'Section Label Style',
      help: metadataFieldHelp.section_label,
      value: metadata.styles.section_label,
      placeholder: d?.section_label ?? null,
    },
    {
      kind: 'part_label',
      label: 'Part Label Style',
      help: metadataFieldHelp.part_label,
      value: metadata.styles.part_label,
      placeholder: d?.part_label ?? null,
    },
    {
      kind: 'page_number',
      label: 'Page Number Style',
      help: metadataFieldHelp.page_number,
      value: metadata.styles.page_number,
      placeholder: stylePlaceholder(d?.page_number, pageNumberFontSizeDefault),
    },
    {
      kind: 'lyrics',
      label: 'Lyrics Style',
      help: metadataFieldHelp.lyrics,
      value: metadata.styles.lyrics,
      placeholder: stylePlaceholder(d?.lyrics, lyricsFontSizeDefault),
    },
    {
      kind: 'notes',
      label: 'Notes Style',
      help: metadataFieldHelp.notes,
      value: metadata.styles.notes,
      placeholder: stylePlaceholder(d?.notes, effectiveLyricsFontSize),
      showFontFamily: false,
    },
    {
      kind: 'chords',
      label: 'Chords Style',
      help: metadataFieldHelp.chords,
      value: metadata.styles.chords,
      placeholder: stylePlaceholder(d?.chords, effectiveLyricsFontSize),
      showFontFamily: false,
    },
    {
      kind: 'note_dash',
      label: 'Note Dash Style',
      help: metadataFieldHelp.note_dash,
      value: metadata.styles.note_dash,
      placeholder: stylePlaceholder(d?.note_dash, effectiveNotesFontSize),
      showFontFamily: false,
    },
  ]

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
            top: container ? 0 : '50%',
            left: container ? 0 : '50%',
            transform: container ? undefined : 'translate(-50%, -50%)',
            background: '#fff',
            border: container ? 'none' : '1px solid #ddd',
            borderRadius: container ? 0 : '6px',
            boxShadow: '0 8px 32px rgba(0,0,0,0.16)',
            zIndex: 1001,
            minWidth: container ? undefined : '420px',
            width: container ? '100%' : undefined,
            height: container ? '100%' : undefined,
            maxWidth: container ? undefined : '90vw',
            maxHeight: container ? undefined : '80vh',
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
            <div style={{ overflowX: 'auto' }}>
              <MetadataStylesTable
                rows={styleRows}
                showHelp={showHelp}
                onChange={setStyle}
              />
            </div>
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
              <MetadataFieldsTableBody
                metadata={metadata}
                defaults={d}
                showHelp={showHelp}
                onTitleChange={(value) => onFieldChange('title', value)}
                setText={setText}
                setNumber={setNumber}
                setYesNo={setYesNo}
                numOrUndef={numOrUndef}
              />
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
