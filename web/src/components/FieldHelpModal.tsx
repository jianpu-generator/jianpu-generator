import * as Dialog from '@radix-ui/react-dialog'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

const fieldLinkStyle: React.CSSProperties = {
  background: 'none',
  border: 'none',
  padding: 0,
  margin: 0,
  font: 'inherit',
  fontSize: '13px',
  color: '#2563eb',
  textDecoration: 'underline',
  textUnderlineOffset: '2px',
  cursor: 'pointer',
  textAlign: 'left',
}

/** Field name rendered as a link; clicking it shows `help` (markdown) in a
 * modal describing every rendering/layout aspect the field affects, so it's
 * clear when a value change won't be visible (e.g. only with certain note
 * types). */
export function FieldLabel({
  label,
  help,
  onShowHelp,
}: {
  label: string
  help: string
  onShowHelp: (label: string, help: string) => void
}) {
  return (
    <button
      type="button"
      style={fieldLinkStyle}
      onClick={() => onShowHelp(label, help)}
    >
      {label}
    </button>
  )
}

const helpDialogContentStyle: React.CSSProperties = {
  position: 'fixed',
  top: '50%',
  left: '50%',
  transform: 'translate(-50%, -50%)',
  background: 'var(--editor-bg, #fff)',
  color: 'var(--fg, #222)',
  border: '1px solid #ddd',
  borderRadius: '6px',
  boxShadow: '0 8px 32px rgba(0,0,0,0.24)',
  zIndex: 1101,
  width: '90%',
  maxWidth: '480px',
  maxHeight: '80vh',
  display: 'flex',
  flexDirection: 'column',
  fontFamily: 'var(--sans, sans-serif)',
}

/** Modal showing the markdown help text for whichever field link was
 * clicked. Rendered on top of the Edit Metadata dialog. */
export function FieldHelpModal({
  content,
  onOpenChange,
}: {
  content: { label: string; help: string } | null
  onOpenChange: (open: boolean) => void
}) {
  return (
    <Dialog.Root open={content !== null} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay
          style={{
            position: 'fixed',
            inset: 0,
            background: 'rgba(0,0,0,0.35)',
            zIndex: 1100,
          }}
        />
        <Dialog.Content style={helpDialogContentStyle}>
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
              {content?.label}
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
          <div
            className="field-help-markdown"
            style={{
              overflowY: 'auto',
              flex: 1,
              padding: '12px 16px',
              fontSize: '13px',
              lineHeight: 1.5,
            }}
          >
            {content && (
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {content.help}
              </ReactMarkdown>
            )}
          </div>
          <style>{`
            .field-help-markdown p, .field-help-markdown ul, .field-help-markdown pre {
              margin: 0 0 10px;
            }
            .field-help-markdown p:last-child, .field-help-markdown ul:last-child, .field-help-markdown pre:last-child {
              margin-bottom: 0;
            }
            .field-help-markdown ul {
              padding-left: 20px;
            }
            .field-help-markdown code {
              background: var(--code-bg, #f0f0f0);
              border-radius: 3px;
              padding: 1px 4px;
              font-family: var(--mono, monospace);
              font-size: 12px;
            }
            .field-help-markdown pre {
              background: var(--code-bg, #f0f0f0);
              border-radius: 4px;
              padding: 8px 10px;
              overflow-x: auto;
            }
            .field-help-markdown pre code {
              background: none;
              padding: 0;
            }
            .field-help-markdown table {
              border-collapse: collapse;
              margin-bottom: 10px;
            }
            .field-help-markdown th, .field-help-markdown td {
              border: 1px solid #ddd;
              padding: 4px 8px;
              font-size: 12px;
            }
          `}</style>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
