import type {
  TextStyleComponent,
  TextStyleFields,
} from '../utils/metadataSource'
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
          aria-label={label}
          placeholder={placeholder}
          style={inputStyle}
          value={value}
          onChange={onChange}
        />
      </td>
    </tr>
  )
}

const styleInputsWrapperStyle: React.CSSProperties = {
  display: 'flex',
  gap: '6px',
}

const styleInputGroupStyle: React.CSSProperties = {
  display: 'flex',
  flexDirection: 'column',
  gap: '2px',
  flex: 1,
  minWidth: 0,
}

const styleInputSubLabelStyle: React.CSSProperties = {
  fontSize: '10px',
  color: '#888',
  whiteSpace: 'nowrap',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
}

const styleComponentOrder: TextStyleComponent[] = [
  'font_size',
  'horizontal_padding_pt',
  'vertical_padding_pt',
]

/** Sub-labels shown above each of a `TextStyleRow`'s three inline inputs —
 * also used to build each input's `aria-label` (`"${label} ${subLabel}"`),
 * so e2e tests can target one specific component by an accessible name
 * rather than positional `nth()` indexing. */
const styleComponentSubLabels: Record<TextStyleComponent, string> = {
  font_size: 'Font Size',
  horizontal_padding_pt: 'H. Padding',
  vertical_padding_pt: 'V. Padding',
}

/** One `<kind> = { font_size: N, horizontal_padding_pt: N,
 * vertical_padding_pt: N }` text-style kind's row: a single label plus
 * three inline number inputs, one per component (see `TextStyleFields`).
 * Replaces what used to be several separate flat-key rows (`title_font_size`,
 * ...) with one row per kind — see `syntax.md`'s "Text styles" section for
 * the unified object syntax this mirrors. `part_label_width_pt` is a plain
 * scalar field (see `NumberFieldRow`), not part of this object, since it's
 * a layout constant rather than a text style component. */
export function TextStyleRow({
  label,
  help,
  onShowHelp,
  value,
  placeholder,
  onChange,
}: FieldRowProps & {
  value: TextStyleFields
  placeholder?: TextStyleFields | null
  onChange: (
    component: TextStyleComponent,
  ) => (e: React.ChangeEvent<HTMLInputElement>) => void
}) {
  return (
    <tr>
      <td style={tdStyle}>
        <FieldLabel label={label} help={help} onShowHelp={onShowHelp} />
      </td>
      <td style={tdStyle}>
        <div style={styleInputsWrapperStyle}>
          {styleComponentOrder.map((component) => (
            <div key={component} style={styleInputGroupStyle}>
              <span style={styleInputSubLabelStyle}>
                {styleComponentSubLabels[component]}
              </span>
              <input
                type="number"
                min="0"
                aria-label={`${label} ${styleComponentSubLabels[component]}`}
                placeholder={
                  placeholder ? String(placeholder[component]) : undefined
                }
                style={inputStyle}
                value={value[component] ?? ''}
                onChange={onChange(component)}
              />
            </div>
          ))}
        </div>
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
