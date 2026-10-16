import { useState } from 'react'
import { dangerButtonStyle, linkRowStyle } from './shareModalStyles'
import { bannerStyle, buttonStyle } from './storageSettingsModalStyles'

interface StopSyncControlProps {
  kind: 'file' | 'folder'
  /** The folder's own name; unused for a file. */
  name: string
  onStop: () => void
}

/** "Stop Sync" with an inline confirmation: ending a link breaks it for
 * everyone who has it, so it takes a second click. Inline rather than a
 * nested dialog, since two open Radix dialogs fight over the focus trap. */
export function StopSyncControl({ kind, name, onStop }: StopSyncControlProps) {
  const [confirming, setConfirming] = useState(false)
  if (!confirming) {
    return (
      <button
        type="button"
        style={dangerButtonStyle}
        data-testid="share-modal-stop-sync"
        onClick={() => setConfirming(true)}
      >
        Stop Sync
      </button>
    )
  }
  return (
    <div
      style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}
      data-testid="share-modal-stop-confirm"
    >
      <div style={bannerStyle}>
        Stop sharing {kind === 'folder' ? `"${name}"` : 'this file'}? The link
        stops working for everyone who has it. Your{' '}
        {kind === 'folder' ? 'files are' : 'file is'} not affected, and syncing
        again brings the same link back.
      </div>
      <div style={linkRowStyle}>
        <button
          type="button"
          style={{ ...buttonStyle, flex: 1 }}
          data-testid="share-modal-cancel-stop-sync"
          onClick={() => setConfirming(false)}
        >
          Cancel
        </button>
        <button
          type="button"
          style={{ ...dangerButtonStyle, flex: 1 }}
          data-testid="share-modal-confirm-stop-sync"
          onClick={() => {
            setConfirming(false)
            onStop()
          }}
        >
          Stop Sync
        </button>
      </div>
    </div>
  )
}
