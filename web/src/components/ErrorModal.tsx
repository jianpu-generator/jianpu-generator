import * as Dialog from '@radix-ui/react-dialog'

export interface ErrorModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  message: string
  stack?: string
}

const overlayStyle: React.CSSProperties = {
  position: 'fixed',
  inset: 0,
  background: 'rgba(0,0,0,0.35)',
  zIndex: 1000,
}

const contentStyle: React.CSSProperties = {
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
}

const headerStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  padding: '12px 16px',
  borderBottom: '1px solid #eee',
}

const bodyStyle: React.CSSProperties = {
  overflowY: 'auto',
  flex: 1,
  padding: '16px',
  fontSize: '13px',
  display: 'flex',
  flexDirection: 'column',
  gap: '12px',
}

const stackStyle: React.CSSProperties = {
  margin: 0,
  padding: '8px 10px',
  borderRadius: '4px',
  background: '#f5f5f5',
  border: '1px solid #eee',
  fontSize: '11px',
  whiteSpace: 'pre-wrap',
  wordBreak: 'break-word',
  color: '#555',
}

export function ErrorModal({
  open,
  onOpenChange,
  title,
  message,
  stack,
}: ErrorModalProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay style={overlayStyle} />
        <Dialog.Content data-testid="error-modal" style={contentStyle}>
          <div style={headerStyle}>
            <Dialog.Title
              style={{
                margin: 0,
                fontSize: '14px',
                fontWeight: 600,
                color: '#b00020',
              }}
            >
              {title}
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
          <div style={bodyStyle}>
            <p style={{ margin: 0 }} data-testid="error-modal-message">
              {message}
            </p>
            {stack ? (
              <pre style={stackStyle} data-testid="error-modal-stack">
                {stack}
              </pre>
            ) : null}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
