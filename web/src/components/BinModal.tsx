import * as Dialog from '@radix-ui/react-dialog'
import { CounterClockwiseClockIcon } from '@radix-ui/react-icons'
import { useEffect } from 'react'
import { displayFileName } from '../fileStore'

export interface BinModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  binNames: string[]
  onRestore: (name: string) => void
  restoringName?: string | null
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
  minWidth: '320px',
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
  padding: '8px 0',
}

/** Swaps a restore button's icon for the shared spinner while `pending`. */
function RestoreIcon({ pending }: { pending: boolean }) {
  return pending ? (
    <span className="file-tab-bar-spinner" aria-hidden="true" />
  ) : (
    <CounterClockwiseClockIcon aria-hidden="true" />
  )
}

/** Auto-closes once the last entry has been restored — otherwise the modal
 * would sit open with an empty list, its overlay still blocking interaction
 * with the rest of the page. */
export function BinModal({
  open,
  onOpenChange,
  binNames,
  onRestore,
  restoringName = null,
}: BinModalProps) {
  useEffect(() => {
    if (open && binNames.length === 0) onOpenChange(false)
  }, [open, binNames.length, onOpenChange])

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay style={overlayStyle} />
        <Dialog.Content data-testid="bin-modal" style={contentStyle}>
          <div style={headerStyle}>
            <Dialog.Title
              style={{ margin: 0, fontSize: '14px', fontWeight: 600 }}
            >
              Bin
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
            <div className="file-tab-bar-bin-items" role="menu">
              {binNames.map((name) => (
                <div key={name} className="file-tab-bar-bin-item">
                  <span className="file-tab-bar-bin-name">
                    {displayFileName(name)}
                  </span>
                  <button
                    type="button"
                    role="menuitem"
                    className="file-tab-bar-restore"
                    aria-label={`Restore ${name}`}
                    onClick={() => onRestore(name)}
                    disabled={restoringName === name}
                  >
                    <RestoreIcon pending={restoringName === name} />
                  </button>
                </div>
              ))}
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
