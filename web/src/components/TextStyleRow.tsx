import type {
  FontFamilyValue,
  TextStyleBooleanComponent,
  TextStyleComponent,
  TextStyleFields,
  TextStyleNumericComponent,
} from '../utils/metadataSource'
import { fontFamilyValues } from '../utils/metadataSource'
import { FieldLabel } from './FieldHelpModal'
import { type FieldRowProps, NumberStepper, tdStyle } from './MetadataFieldRows'

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

const styleNumericComponentOrder: TextStyleNumericComponent[] = [
  'font_size',
  'horizontal_padding_pt',
  'vertical_padding_pt',
]

const styleBooleanComponentOrder: TextStyleBooleanComponent[] = [
  'bold',
  'italic',
  'underline',
]

/** Sub-labels shown above each of a `TextStyleRow`'s three inline number
 * inputs — also used to build each input's `aria-label` (`"${label}
 * ${subLabel}"`), so e2e tests can target one specific component by an
 * accessible name rather than positional `nth()` indexing. */
const styleNumericComponentSubLabels: Record<
  TextStyleNumericComponent,
  string
> = {
  font_size: 'Font Size',
  horizontal_padding_pt: 'H. Padding',
  vertical_padding_pt: 'V. Padding',
}

/** Full label (used for each toggle button's `aria-label`, as
 * `"${label} ${styleBooleanComponentLabels[component]}"`) and single-letter
 * glyph (the button's visible content, styled to preview the effect it
 * toggles) for each of a `TextStyleRow`'s three style-flag toggle buttons. */
const styleBooleanComponentLabels: Record<TextStyleBooleanComponent, string> = {
  bold: 'Bold',
  italic: 'Italic',
  underline: 'Underline',
}
const styleBooleanComponentGlyphs: Record<TextStyleBooleanComponent, string> = {
  bold: 'B',
  italic: 'I',
  underline: 'U',
}

const styleToggleButtonStyle: React.CSSProperties = {
  width: '22px',
  height: '22px',
  lineHeight: 1,
  fontSize: '13px',
  padding: 0,
  border: '1px solid #cbd5e0',
  borderRadius: '3px',
  background: '#fff',
  color: '#444',
  cursor: 'pointer',
}

const styleToggleButtonPressedStyle: React.CSSProperties = {
  ...styleToggleButtonStyle,
  background: '#e2e8f0',
  borderColor: '#94a3b8',
  color: '#1a202c',
}

const styleToggleGlyphStyle: Record<
  TextStyleBooleanComponent,
  React.CSSProperties
> = {
  bold: { fontWeight: 'bold' },
  italic: { fontStyle: 'italic' },
  underline: { textDecoration: 'underline' },
}

const styleTogglesWrapperStyle: React.CSSProperties = {
  display: 'flex',
  gap: '4px',
  alignItems: 'flex-end',
}

/** Display label for each `font_family` option, shown in the `<select>`
 * (see `TextStyleRow`'s `showFontFamily` prop). */
const fontFamilyOptionLabels: Record<FontFamilyValue, string> = {
  serif: 'Serif',
  sans_serif: 'Sans Serif',
  monospace: 'Monospace',
}

const fontFamilySelectStyle: React.CSSProperties = {
  height: '22px',
  fontSize: '11px',
  border: '1px solid #cbd5e0',
  borderRadius: '3px',
  background: '#fff',
  color: '#444',
}

/** One `B`/`I`/`U` toggle button for a single boolean style component:
 * `checked` reflects the effective value (the field's own override, falling
 * back to its default when unset — same fallback `NumberStepper` shows via
 * its greyed-out `placeholder`), and clicking always writes the opposite as
 * an explicit `yes`/`no` (mirrors `CheckboxFieldRow`, which never exposes an
 * "unset" state in its UI either). */
function StyleToggleButton({
  component,
  checked,
  ariaLabel,
  onClick,
}: {
  component: TextStyleBooleanComponent
  checked: boolean
  ariaLabel: string
  onClick: () => void
}) {
  return (
    <button
      type="button"
      aria-label={ariaLabel}
      aria-pressed={checked}
      style={checked ? styleToggleButtonPressedStyle : styleToggleButtonStyle}
      onClick={onClick}
    >
      <span style={styleToggleGlyphStyle[component]}>
        {styleBooleanComponentGlyphs[component]}
      </span>
    </button>
  )
}

/** One `<kind> = { font_size: N, horizontal_padding_pt: N,
 * vertical_padding_pt: N, bold: yes/no, italic: yes/no, underline: yes/no }`
 * text-style kind's row: a single label, three inline number inputs, and
 * three bold/italic/underline toggle buttons, one per component (see
 * `TextStyleFields`). Replaces what used to be several separate flat-key
 * rows (`title_font_size`, ...) with one row per kind — see `syntax.md`'s
 * "Text styles" section for the unified object syntax this mirrors.
 * `part_label_width_pt` is a plain scalar field (see `NumberFieldRow`), not
 * part of this object, since it's a layout constant rather than a text
 * style component. */
export function TextStyleRow({
  label,
  help,
  onShowHelp,
  value,
  placeholder,
  onChange,
  showFontFamily = true,
}: FieldRowProps & {
  value: TextStyleFields
  placeholder?: TextStyleFields | null
  onChange: (component: TextStyleComponent) => (value: string) => void
  /** `false` for `notes`/`chords`/`note_dash`, whose glyph widths are
   * layout-measured in a fixed monospace font — `font_family` isn't
   * accepted on those kinds (see `syntax.md`). */
  showFontFamily?: boolean
}) {
  return (
    <tr>
      <td style={tdStyle}>
        <FieldLabel label={label} help={help} onShowHelp={onShowHelp} />
      </td>
      <td style={tdStyle}>
        <div style={styleInputsWrapperStyle}>
          {styleNumericComponentOrder.map((component) => (
            <div key={component} style={styleInputGroupStyle}>
              <span style={styleInputSubLabelStyle}>
                {styleNumericComponentSubLabels[component]}
              </span>
              <NumberStepper
                value={value[component] ?? ''}
                defaultValue={placeholder ? placeholder[component] : null}
                min={0}
                aria-label={`${label} ${styleNumericComponentSubLabels[component]}`}
                placeholder={
                  placeholder ? String(placeholder[component]) : undefined
                }
                onChange={onChange(component)}
              />
            </div>
          ))}
          <div style={styleTogglesWrapperStyle}>
            {styleBooleanComponentOrder.map((component) => {
              const checked =
                value[component] ?? placeholder?.[component] ?? false
              return (
                <StyleToggleButton
                  key={component}
                  component={component}
                  checked={checked}
                  ariaLabel={`${label} ${styleBooleanComponentLabels[component]}`}
                  onClick={() => onChange(component)(checked ? 'no' : 'yes')}
                />
              )
            })}
          </div>
          {showFontFamily && (
            <div style={styleInputGroupStyle}>
              <span style={styleInputSubLabelStyle}>Font</span>
              <select
                aria-label={`${label} Font Family`}
                style={fontFamilySelectStyle}
                value={value.font_family ?? placeholder?.font_family ?? ''}
                onChange={(e) => onChange('font_family')(e.target.value)}
              >
                {fontFamilyValues.map((option) => (
                  <option key={option} value={option}>
                    {fontFamilyOptionLabels[option]}
                  </option>
                ))}
              </select>
            </div>
          )}
        </div>
      </td>
    </tr>
  )
}
