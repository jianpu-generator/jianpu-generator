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

const numberStepperWrapperStyle: React.CSSProperties = {
  display: 'flex',
  width: '100%',
}

const numberStepperInputStyle: React.CSSProperties = {
  ...inputStyle,
  minWidth: 0,
  flex: 1,
  borderTopRightRadius: 0,
  borderBottomRightRadius: 0,
}

const numberStepperButtonsStyle: React.CSSProperties = {
  display: 'flex',
  flexShrink: 0,
}

const numberStepperButtonStyle: React.CSSProperties = {
  width: '18px',
  height: '20px',
  lineHeight: 1,
  fontSize: '12px',
  padding: 0,
  border: '1px solid #cbd5e0',
  borderLeft: 'none',
  color: '#444',
  cursor: 'pointer',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
}

interface NumberStepperProps {
  value: number | ''
  defaultValue: number | null | undefined
  min: number
  step?: number
  'aria-label': string
  placeholder?: string
  onChange: (value: string) => void
}

/** `[input][-][+]` numeric field: always-visible stepper buttons (native
 * spinner arrows are hover-only in some browsers), and stepping from an
 * empty field applies the field's default rather than starting from `0`
 * (native `<input type="number">` always steps from `0` when empty). See
 * `HANDOFF-text-style-metadata.md`-adjacent context: `defaultValue` is the
 * same value shown today as the input's greyed-out `placeholder`. */
function NumberStepper({
  value,
  defaultValue,
  min,
  step = 1,
  'aria-label': ariaLabel,
  placeholder,
  onChange,
}: NumberStepperProps) {
  const stepBy = (delta: number) => {
    const base = value === '' ? (defaultValue ?? min) : value
    onChange(String(Math.max(min, base + delta)))
  }
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'ArrowUp') {
      e.preventDefault()
      stepBy(step)
    } else if (e.key === 'ArrowDown') {
      e.preventDefault()
      stepBy(-step)
    }
  }
  return (
    <div style={numberStepperWrapperStyle}>
      <input
        type="number"
        min={min}
        aria-label={ariaLabel}
        placeholder={placeholder}
        style={numberStepperInputStyle}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
      />
      <div style={numberStepperButtonsStyle}>
        <button
          type="button"
          className="number-stepper-button"
          aria-label={`${ariaLabel} decrease`}
          style={numberStepperButtonStyle}
          onClick={() => stepBy(-step)}
        >
          −
        </button>
        <button
          type="button"
          className="number-stepper-button"
          aria-label={`${ariaLabel} increase`}
          style={{ ...numberStepperButtonStyle, borderLeft: 'none' }}
          onClick={() => stepBy(step)}
        >
          +
        </button>
      </div>
    </div>
  )
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
  onChange: (value: string) => void
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
          onChange={(e) => onChange(e.target.value)}
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
  onChange: (value: string) => void
}) {
  return (
    <tr>
      <td style={tdStyle}>
        <FieldLabel label={label} help={help} onShowHelp={onShowHelp} />
      </td>
      <td style={tdStyle}>
        <NumberStepper
          value={value}
          defaultValue={placeholder != null ? Number(placeholder) : null}
          min={1}
          aria-label={label}
          placeholder={placeholder}
          onChange={onChange}
        />
      </td>
    </tr>
  )
}

const styleInputsWrapperStyle: React.CSSProperties = {
  display: 'flex',
  gap: '4px',
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
  onChange: (component: TextStyleComponent) => (value: string) => void
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
              <NumberStepper
                value={value[component] ?? ''}
                defaultValue={placeholder ? placeholder[component] : null}
                min={0}
                aria-label={`${label} ${styleComponentSubLabels[component]}`}
                placeholder={
                  placeholder ? String(placeholder[component]) : undefined
                }
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
