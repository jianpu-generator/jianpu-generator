import type { TextStyleDefaults } from '../utils/metadataDefaults'
import type {
  FontFamilyValue,
  TextStyleBooleanComponent,
  TextStyleComponent,
  TextStyleFields,
  TextStyleKind,
  TextStyleNumericComponent,
} from '../utils/metadataSource'
import { fontFamilyValues } from '../utils/metadataSource'
import { FieldLabel } from './FieldHelpModal'
import { NumberStepper } from './MetadataFieldRows'

export interface StyleRowSpec {
  kind: TextStyleKind
  label: string
  help: string
  value: TextStyleFields
  placeholder: TextStyleDefaults | null
  /** `false` for `notes`/`chords`/`note_dash`, whose glyph widths are
   * layout-measured in a fixed monospace font — `font_family` isn't
   * accepted on those kinds (see `syntax.md`). */
  showFontFamily?: boolean
}

const thStyle: React.CSSProperties = {
  padding: '6px 4px',
  textAlign: 'center',
  fontWeight: 600,
  fontSize: '11px',
  color: '#444',
  borderBottom: '2px solid #ddd',
  background: '#f5f5f5',
  whiteSpace: 'nowrap',
}

const thLabelStyle: React.CSSProperties = {
  ...thStyle,
  textAlign: 'left',
  padding: '6px 10px',
}

const tdStyle: React.CSSProperties = {
  padding: '4px',
  borderBottom: '1px solid #eee',
  verticalAlign: 'middle',
}

const tdLabelStyle: React.CSSProperties = {
  ...tdStyle,
  padding: '6px 10px',
  fontSize: '13px',
  whiteSpace: 'nowrap',
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

/** Sub-labels shown as this table's column headers — also used to build
 * each numeric input's `aria-label` (`"${rowLabel} ${subLabel}"`), so e2e
 * tests can target one specific component by an accessible name rather
 * than positional `nth()` indexing. */
const styleNumericComponentSubLabels: Record<
  TextStyleNumericComponent,
  string
> = {
  font_size: 'Font Size',
  horizontal_padding_pt: 'H. Padding',
  vertical_padding_pt: 'V. Padding',
}

/** Full label (used for each toggle button's `aria-label`, as
 * `"${rowLabel} ${styleBooleanComponentLabels[component]}"`) and
 * single-letter glyph (the button's visible content, styled to preview the
 * effect it toggles) for each boolean style-flag column. */
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

/** Display label for each `font_family` option, shown in the `<select>`
 * (see `StyleRowSpec.showFontFamily`). */
const fontFamilyOptionLabels: Record<FontFamilyValue, string> = {
  serif: 'Serif',
  sans_serif: 'Sans Serif',
  monospace: 'Monospace',
}

const fontFamilySelectStyle: React.CSSProperties = {
  width: '100%',
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

function StyleTableRow({
  row,
  showHelp,
  onChange,
}: {
  row: StyleRowSpec
  showHelp: (label: string, help: string) => void
  onChange: (component: TextStyleComponent) => (value: string) => void
}) {
  const { label, help, value, placeholder, showFontFamily = true } = row
  return (
    <tr>
      <td style={tdLabelStyle}>
        <FieldLabel label={label} help={help} onShowHelp={showHelp} />
      </td>
      {styleNumericComponentOrder.map((component) => (
        <td key={component} style={tdStyle}>
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
        </td>
      ))}
      {styleBooleanComponentOrder.map((component) => {
        const checked = value[component] ?? placeholder?.[component] ?? false
        return (
          <td key={component} style={{ ...tdStyle, textAlign: 'center' }}>
            <StyleToggleButton
              component={component}
              checked={checked}
              ariaLabel={`${label} ${styleBooleanComponentLabels[component]}`}
              onClick={() => onChange(component)(checked ? 'no' : 'yes')}
            />
          </td>
        )
      })}
      <td style={tdStyle}>
        {showFontFamily && (
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
        )}
      </td>
    </tr>
  )
}

/** The "Text Styles" half of `EditMetadataModal`'s field table: one row per
 * `<kind> = { font_size: N, horizontal_padding_pt: N, vertical_padding_pt:
 * N, bold: yes/no, italic: yes/no, underline: yes/no, font_family: ... }`
 * text-style kind (see `TextStyleFields`), with the component sub-labels
 * (Font Size/H. Padding/V. Padding/B/I/U/Font) hoisted into a single header
 * row instead of repeating per kind. Rendered as its own `<table>`,
 * separate from the plain label/value table for non-style fields (see
 * `MetadataFieldsTableBody`). */
export function MetadataStylesTable({
  rows,
  showHelp,
  onChange,
}: {
  rows: StyleRowSpec[]
  showHelp: (label: string, help: string) => void
  onChange: (
    kind: TextStyleKind,
  ) => (component: TextStyleComponent) => (value: string) => void
}) {
  return (
    <table
      style={{
        width: '100%',
        borderCollapse: 'collapse',
      }}
    >
      <colgroup>
        <col style={{ width: '22%' }} />
        <col style={{ width: '13%' }} />
        <col style={{ width: '13%' }} />
        <col style={{ width: '13%' }} />
        <col style={{ width: '8%' }} />
        <col style={{ width: '8%' }} />
        <col style={{ width: '8%' }} />
        <col style={{ width: '15%' }} />
      </colgroup>
      <thead>
        <tr>
          <th style={thLabelStyle}>Text Style</th>
          <th style={thStyle}>Font Size</th>
          <th style={thStyle}>H. Padding</th>
          <th style={thStyle}>V. Padding</th>
          <th style={thStyle}>B</th>
          <th style={thStyle}>I</th>
          <th style={thStyle}>U</th>
          <th style={thStyle}>Font</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <StyleTableRow
            key={row.kind}
            row={row}
            showHelp={showHelp}
            onChange={onChange(row.kind)}
          />
        ))}
      </tbody>
    </table>
  )
}
