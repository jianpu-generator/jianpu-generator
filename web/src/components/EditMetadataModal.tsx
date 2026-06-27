import * as Dialog from '@radix-ui/react-dialog'
import type { MetadataKey, ParsedMetadataFields } from '../utils/metadataSource'

export interface EditMetadataModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  metadata: ParsedMetadataFields
  onFieldChange: (key: MetadataKey, value: string | null) => void
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
}: EditMetadataModalProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay
          style={{
            position: 'fixed',
            inset: 0,
            background: 'rgba(0,0,0,0.35)',
            zIndex: 1000,
          }}
        />
        <Dialog.Content
          data-testid="edit-metadata-modal"
          style={{
            position: 'fixed',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            background: '#fff',
            border: '1px solid #ddd',
            borderRadius: '6px',
            boxShadow: '0 8px 32px rgba(0,0,0,0.16)',
            zIndex: 1001,
            minWidth: '420px',
            maxWidth: '90vw',
            maxHeight: '80vh',
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
                      style={inputStyle}
                      value={metadata.rowHeight ?? ''}
                      onChange={(e) =>
                        onFieldChange(
                          'row height',
                          e.target.value === '' ? null : e.target.value,
                        )
                      }
                    />
                  </td>
                </tr>
                <tr>
                  <td style={tdStyle}>Max Columns</td>
                  <td style={tdStyle}>
                    <input
                      type="number"
                      min="1"
                      style={inputStyle}
                      value={metadata.maxColumns ?? ''}
                      onChange={(e) =>
                        onFieldChange(
                          'max columns',
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
                      style={inputStyle}
                      value={metadata.labelWidth ?? ''}
                      onChange={(e) =>
                        onFieldChange(
                          'label width',
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
                      style={inputStyle}
                      value={metadata.noteNumberWidth ?? ''}
                      onChange={(e) =>
                        onFieldChange(
                          'note number width',
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
