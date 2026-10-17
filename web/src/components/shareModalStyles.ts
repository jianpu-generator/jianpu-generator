import type { CSSProperties } from 'react'
import { buttonStyle } from './storageSettingsModalStyles'

export const dangerButtonStyle: CSSProperties = {
  ...buttonStyle,
  width: '100%',
  padding: '8px 12px',
  fontSize: '13px',
  border: '1px solid #e0a0a0',
  background: '#fff',
  color: '#b00020',
}

export const helpTextStyle: CSSProperties = {
  margin: 0,
  color: '#666',
  fontSize: '12px',
}

export const linkRowStyle: CSSProperties = {
  display: 'flex',
  gap: '8px',
}

export const linkFieldStyle: CSSProperties = {
  flex: 1,
  fontFamily: 'inherit',
  fontSize: '12px',
  padding: '6px 8px',
  borderRadius: '4px',
  border: '1px solid #cbd5e0',
  background: '#f9f9f9',
  color: 'inherit',
}
