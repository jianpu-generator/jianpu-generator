import * as Dialog from '@radix-ui/react-dialog'
import { useEffect, useState } from 'react'
import type {
  MetadataEdit,
  MetadataFields,
  TextStyleComponentValue,
  TextStyleDefaults,
  TextStyleKind,
} from '../jianpuWasm'
import {
  loadMetadataDefaults,
  type MetadataDefaults,
} from '../utils/metadataDefaults'
import { metadataFieldHelp } from '../utils/metadataFieldHelp'
import { useFontSizeDefaults } from '../utils/useFontSizeDefaults'
import { FieldHelpModal } from './FieldHelpModal'
import { MetadataFieldsTableBody } from './MetadataFieldsTableBody'
import type { StyleRowSpec } from './MetadataStylesTable'
import { MetadataStylesTable } from './MetadataStylesTable'

export interface EditMetadataModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** `null` until the wasm-side parse is available (see
   * `useMetadataFields`); the modal shows no fields until then. */
  metadata: MetadataFields | null
  onFieldChange: (edit: MetadataEdit) => void
  /** Element to confine the modal to (e.g. the editor pane), so it doesn't
   * cover the preview pane. Falls back to viewport-centered when null. */
  container?: HTMLElement | null
}

const styleRowLabels: Record<TextStyleKind, string> = {
  title: 'Title Style',
  subtitle: 'Subtitle Style',
  author: 'Author Style',
  sequence: 'Sequence Style',
  'part-legend': 'Part Legend Style',
  'measure-number': 'Measure Number Style',
  'section-label': 'Section Label Style',
  'part-label': 'Part Label Style',
  'page-number': 'Page Number Style',
  lyrics: 'Lyrics Style',
  notes: 'Notes Style',
  chords: 'Chords Style',
  'note-dash': 'Note Dash Style',
}

function styleDefaultsByKind(
  d: MetadataDefaults,
): Record<TextStyleKind, TextStyleDefaults> {
  return {
    title: d.title,
    subtitle: d.subtitle,
    author: d.author,
    sequence: d.sequence,
    'part-legend': d.partLegend,
    'measure-number': d.measureNumber,
    'section-label': d.sectionLabel,
    'part-label': d.partLabel,
    'page-number': d.pageNumber,
    lyrics: d.lyrics,
    notes: d.notes,
    chords: d.chords,
    'note-dash': d.noteDash,
  }
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

  const effectiveRowHeight = metadata?.rowHeight ?? defaults?.rowHeight ?? null
  const {
    lyricsFontSizeDefault,
    titleFontSizeDefault,
    subtitleFontSizeDefault,
    authorFontSizeDefault,
    partLegendFontSizeDefault,
    pageNumberFontSizeDefault,
  } = useFontSizeDefaults(effectiveRowHeight)

  const d = defaults
  const styleOf = (kind: TextStyleKind) =>
    metadata?.styles.find((entry) => entry.kind === kind)?.fields

  // notes/chords styles' font_size default to the *effective* lyrics font
  // size — either the explicit override or its own row_height-derived
  // default; note_dash's font_size then defaults to that effective notes
  // font size, one level further down the cascade (see `syntax.md`'s
  // "Text styles" defaults table).
  const effectiveLyricsFontSize =
    styleOf('lyrics')?.fontSize ?? lyricsFontSizeDefault
  const effectiveNotesFontSize =
    styleOf('notes')?.fontSize ?? effectiveLyricsFontSize

  // The kinds whose real `font_size` default isn't a flat constant but
  // depends on `row_height` (or on `lyrics`' font size) — these override
  // the static `d.<kind>` snapshot with a live value (see
  // `useFontSizeDefaults` above), falling back to the snapshot while the
  // live value isn't known yet.
  const liveFontSizeDefaults: Partial<Record<TextStyleKind, number | null>> = {
    title: titleFontSizeDefault,
    subtitle: subtitleFontSizeDefault,
    author: authorFontSizeDefault,
    'part-legend': partLegendFontSizeDefault,
    'page-number': pageNumberFontSizeDefault,
    lyrics: lyricsFontSizeDefault,
    notes: effectiveLyricsFontSize,
    chords: effectiveLyricsFontSize,
    'note-dash': effectiveNotesFontSize,
  }

  const stylePlaceholder = (kind: TextStyleKind): TextStyleDefaults | null => {
    if (!d) return null
    const base = styleDefaultsByKind(d)[kind]
    const liveFontSize = liveFontSizeDefaults[kind]
    return liveFontSize == null ? base : { ...base, fontSize: liveFontSize }
  }

  const styleRows: StyleRowSpec[] = (metadata?.styles ?? []).map(
    ({ kind, fields }) => ({
      kind,
      label: styleRowLabels[kind],
      help: metadataFieldHelp[kind],
      value: fields,
      placeholder: stylePlaceholder(kind),
    }),
  )

  const setStyle = (kind: TextStyleKind) => (value: TextStyleComponentValue) =>
    onFieldChange({ tag: 'style', val: { kind, value } })

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
          {metadata && (
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
                  onFieldChange={onFieldChange}
                />
              </table>
            </div>
          )}
          <FieldHelpModal
            content={helpContent}
            onOpenChange={(open) => !open && setHelpContent(null)}
          />
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
