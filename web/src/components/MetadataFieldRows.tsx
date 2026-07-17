import { FieldLabel } from './FieldHelpModal'

export const tdStyle: React.CSSProperties = {
  padding: '6px 10px',
  borderBottom: '1px solid #eee',
  verticalAlign: 'middle',
  fontSize: '13px',
}

export const inputStyle: React.CSSProperties = {
  fontSize: '12px',
  fontFamily: 'var(--mono, monospace)',
  border: '1px solid #cbd5e0',
  borderRadius: '3px',
  padding: '2px 6px',
  width: '100%',
  boxSizing: 'border-box',
}

interface FieldRowProps {
  label: string
  help: string
  onShowHelp: (label: string, help: string) => void
}

export function TextFieldRow({
  label,
  help,
  onShowHelp,
  value,
  placeholder,
  onChange,
}: FieldRowProps & {
  value: string
  placeholder?: string
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void
}) {
  return (
    <tr>
      <td style={tdStyle}>
        <FieldLabel label={label} help={help} onShowHelp={onShowHelp} />
      </td>
      <td style={tdStyle}>
        <input
          type="text"
          placeholder={placeholder}
          style={inputStyle}
          value={value}
          onChange={onChange}
        />
      </td>
    </tr>
  )
}

export function NumberFieldRow({
  label,
  help,
  onShowHelp,
  value,
  placeholder,
  onChange,
}: FieldRowProps & {
  value: number | ''
  placeholder?: string
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void
}) {
  return (
    <tr>
      <td style={tdStyle}>
        <FieldLabel label={label} help={help} onShowHelp={onShowHelp} />
      </td>
      <td style={tdStyle}>
        <input
          type="number"
          min="1"
          placeholder={placeholder}
          style={inputStyle}
          value={value}
          onChange={onChange}
        />
      </td>
    </tr>
  )
}

export function CheckboxFieldRow({
  label,
  help,
  onShowHelp,
  checked,
  onChange,
}: FieldRowProps & {
  checked: boolean
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void
}) {
  return (
    <tr>
      <td style={tdStyle}>
        <FieldLabel label={label} help={help} onShowHelp={onShowHelp} />
      </td>
      <td style={tdStyle}>
        <input type="checkbox" checked={checked} onChange={onChange} />
      </td>
    </tr>
  )
}
