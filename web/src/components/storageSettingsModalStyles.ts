export const overlayStyle: React.CSSProperties = {
  position: 'fixed',
  inset: 0,
  background: 'rgba(0,0,0,0.35)',
  zIndex: 1000,
}

export const contentStyle: React.CSSProperties = {
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

export const headerStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  padding: '12px 16px',
  borderBottom: '1px solid #eee',
}

export const bodyStyle: React.CSSProperties = {
  overflowY: 'auto',
  flex: 1,
  padding: '16px',
  fontSize: '13px',
  display: 'flex',
  flexDirection: 'column',
  gap: '12px',
}

export const optionRowStyle: React.CSSProperties = {
  display: 'flex',
  gap: '12px',
}

export const backendButtonStyle: React.CSSProperties = {
  flex: 1,
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
  gap: '8px',
  padding: '16px 12px',
  borderRadius: '8px',
  border: '1px solid #ddd',
  background: '#fff',
  cursor: 'pointer',
  fontSize: '13px',
  fontFamily: 'inherit',
  color: 'inherit',
}

export const backendButtonSelectedStyle: React.CSSProperties = {
  ...backendButtonStyle,
  border: '1px solid var(--accent-selected-border)',
  background: 'var(--accent-selected-bg)',
  color: 'var(--accent-selected-text)',
}

export const bannerStyle: React.CSSProperties = {
  padding: '8px 10px',
  borderRadius: '4px',
  fontSize: '12px',
  background: '#fff4e5',
  border: '1px solid #f0c987',
  color: '#7a4b00',
}

export const buttonStyle: React.CSSProperties = {
  fontSize: '12px',
  padding: '4px 10px',
  borderRadius: '4px',
  border: '1px solid #cbd5e0',
  background: '#f5f5f5',
  cursor: 'pointer',
}

/** Shared accent-blue "primary action" button style -- originally defined
 * in `ShareModal.tsx` alone, hoisted here so `StorageSettingsModal`'s "Sign
 * in with GitHub" button matches it too instead of the two dialogs each
 * styling the same action differently (see `HANDOFF-account-chip.md`). */
export const primaryButtonStyle: React.CSSProperties = {
  ...buttonStyle,
  width: '100%',
  padding: '8px 12px',
  fontSize: '13px',
  border: '1px solid var(--accent-selected-border)',
  background: 'var(--accent-selected-bg)',
  color: 'var(--accent-selected-text)',
}

/** Plain "signed in as @login" status line -- what both dialogs render once
 * connected, now that sign-out lives only in the header's account chip
 * (see `HANDOFF-account-chip.md`). */
export const statusLineStyle: React.CSSProperties = {
  margin: 0,
  display: 'flex',
  alignItems: 'center',
  gap: '4px',
  color: '#444',
  fontSize: '13px',
}
