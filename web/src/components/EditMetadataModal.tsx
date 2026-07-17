import * as Dialog from '@radix-ui/react-dialog'
import { useEffect, useState } from 'react'
import type { MetadataDefaults } from '../utils/metadataDefaults'
import {
  defaultLyricsFontSize,
  loadMetadataDefaults,
} from '../utils/metadataDefaults'
import type { MetadataKey, ParsedMetadataFields } from '../utils/metadataSource'

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

const tdStyle: React.CSSProperties = {
  padding: '6px 10px',
  borderBottom: '1px solid #eee',
  verticalAlign: 'middle',
  fontSize: '13px',
}

const inputStyle: React.CSSProperties = {
  fontSize: '12px',
  fontFamily: 'var(--mono, monospace)',
  border: '1px solid #cbd5e0',
  borderRadius: '3px',
  padding: '2px 6px',
  width: '100%',
  boxSizing: 'border-box',
}

export function EditMetadataModal({
  open,
  onOpenChange,
  metadata,
  onFieldChange,
  container,
}: EditMetadataModalProps) {
  const [defaults, setDefaults] = useState<MetadataDefaults | null>(null)
  const [lyricsFontSizeDefault, setLyricsFontSizeDefault] = useState<
    number | null
  >(null)

  useEffect(() => {
    loadMetadataDefaults().then(setDefaults)
  }, [])

  const effectiveRowHeight = metadata.row_height ?? defaults?.row_height ?? null
  useEffect(() => {
    if (effectiveRowHeight === null) return
    defaultLyricsFontSize(effectiveRowHeight).then(setLyricsFontSizeDefault)
  }, [effectiveRowHeight])

  const d = defaults

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
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
                <tr>
                  <td style={tdStyle}>Title *</td>
                  <td style={tdStyle}>
                    <input
                      type="text"
                      style={inputStyle}
                      value={metadata.title}
                      onChange={(e) => onFieldChange('title', e.target.value)}
                    />
                  </td>
                </tr>
                <tr>
                  <td style={tdStyle}>Subtitle</td>
                  <td style={tdStyle}>
                    <input
                      type="text"
                      style={inputStyle}
                      value={metadata.subtitle ?? ''}
                      onChange={(e) =>
                        onFieldChange(
                          'subtitle',
                          e.target.value === '' ? null : e.target.value,
                        )
                      }
                    />
                  </td>
                </tr>
                <tr>
                  <td style={tdStyle}>Author</td>
                  <td style={tdStyle}>
                    <input
                      type="text"
                      style={inputStyle}
                      value={metadata.author ?? ''}
                      onChange={(e) =>
                        onFieldChange(
                          'author',
                          e.target.value === '' ? null : e.target.value,
                        )
                      }
                    />
                  </td>
                </tr>
                <tr>
                  <td style={tdStyle}>Row Height</td>
                  <td style={tdStyle}>
                    <input
                      type="number"
                      min="1"
                      placeholder={d ? String(d.row_height) : undefined}
                      style={inputStyle}
                      value={metadata.row_height ?? ''}
                      onChange={(e) =>
                        onFieldChange(
                          'row_height',
                          e.target.value === '' ? null : e.target.value,
                        )
                      }
                    />
                  </td>
                </tr>
                <tr>
                  <td style={tdStyle}>Max Measures Per System</td>
                  <td style={tdStyle}>
                    <input
                      type="number"
                      min="1"
                      placeholder={
                        d ? String(d.max_measures_per_system) : undefined
                      }
                      style={inputStyle}
                      value={metadata.max_measures_per_system ?? ''}
                      onChange={(e) =>
                        onFieldChange(
                          'max_measures_per_system',
                          e.target.value === '' ? null : e.target.value,
                        )
                      }
                    />
                  </td>
                </tr>
                <tr>
                  <td style={tdStyle}>Label Width</td>
                  <td style={tdStyle}>
                    <input
                      type="number"
                      min="1"
                      placeholder={d ? String(d.label_width) : undefined}
                      style={inputStyle}
                      value={metadata.label_width ?? ''}
                      onChange={(e) =>
                        onFieldChange(
                          'label_width',
                          e.target.value === '' ? null : e.target.value,
                        )
                      }
                    />
                  </td>
                </tr>
                <tr>
                  <td style={tdStyle}>Note Number Width</td>
                  <td style={tdStyle}>
                    <input
                      type="number"
                      min="1"
                      placeholder={d ? String(d.note_number_width) : undefined}
                      style={inputStyle}
                      value={metadata.note_number_width ?? ''}
                      onChange={(e) =>
                        onFieldChange(
                          'note_number_width',
                          e.target.value === '' ? null : e.target.value,
                        )
                      }
                    />
                  </td>
                </tr>
                <tr>
                  <td style={tdStyle}>Parts List Columns</td>
                  <td style={tdStyle}>
                    <input
                      type="number"
                      min="1"
                      placeholder={d ? String(d.parts_list_columns) : undefined}
                      style={inputStyle}
                      value={metadata.parts_list_columns ?? ''}
                      onChange={(e) =>
                        onFieldChange(
                          'parts_list_columns',
                          e.target.value === '' ? null : e.target.value,
                        )
                      }
                    />
                  </td>
                </tr>
                <tr>
                  <td style={tdStyle}>Lyrics Font Size</td>
                  <td style={tdStyle}>
                    <input
                      type="number"
                      min="1"
                      placeholder={
                        lyricsFontSizeDefault !== null
                          ? String(lyricsFontSizeDefault)
                          : undefined
                      }
                      style={inputStyle}
                      value={metadata.lyrics_font_size ?? ''}
                      onChange={(e) =>
                        onFieldChange(
                          'lyrics_font_size',
                          e.target.value === '' ? null : e.target.value,
                        )
                      }
                    />
                  </td>
                </tr>
                <tr>
                  <td style={tdStyle}>Merge Duplicate Measures Across Parts</td>
                  <td style={tdStyle}>
                    <input
                      type="checkbox"
                      checked={
                        metadata.merge_duplicate_measures_across_parts ??
                        d?.merge_duplicate_measures_across_parts ??
                        true
                      }
                      onChange={(e) =>
                        onFieldChange(
                          'merge_duplicate_measures_across_parts',
                          e.target.checked ? 'yes' : 'no',
                        )
                      }
                    />
                  </td>
                </tr>
                <tr>
                  <td style={tdStyle}>Hide Resting Parts</td>
                  <td style={tdStyle}>
                    <input
                      type="checkbox"
                      checked={
                        metadata.hide_resting_parts ??
                        d?.hide_resting_parts ??
                        true
                      }
                      onChange={(e) =>
                        onFieldChange(
                          'hide_resting_parts',
                          e.target.checked ? 'yes' : 'no',
                        )
                      }
                    />
                  </td>
                </tr>
                <tr>
                  <td style={tdStyle}>Hide System Dividers</td>
                  <td style={tdStyle}>
                    <input
                      type="checkbox"
                      checked={
                        metadata.hide_system_dividers ??
                        d?.hide_system_dividers ??
                        false
                      }
                      onChange={(e) =>
                        onFieldChange(
                          'hide_system_dividers',
                          e.target.checked ? 'yes' : 'no',
                        )
                      }
                    />
                  </td>
                </tr>
                <tr>
                  <td style={tdStyle}>Section Label Offset (x y)</td>
                  <td style={tdStyle}>
                    <input
                      type="text"
                      placeholder={
                        d
                          ? `${d.section_label_offset_x} ${d.section_label_offset_y}`
                          : undefined
                      }
                      style={inputStyle}
                      value={metadata.section_label_offset ?? ''}
                      onChange={(e) =>
                        onFieldChange(
                          'section_label_offset',
                          e.target.value === '' ? null : e.target.value,
                        )
                      }
                    />
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
