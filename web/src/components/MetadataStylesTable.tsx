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

/** Whether a `bold`/`italic` toggle actually changes anything for a given
 * `font_family` role — `underline` is `text-decoration`, not a font face, so
 * it always works and isn't checked here.
 *
 * - `serif` (Zhuque Fangsong) and `sans_serif` (Source Han Sans SC) each
 *   bundle only their Regular file (see `fonts/fonts.json`), rendered via a
 *   dedicated `@font-face` pinned to that one file (`injectFontFaces.ts`),
 *   with `font-synthesis: none` set app-wide (`index.css`) — so there's no
 *   real bold/italic face for the browser to pick, and no synthesis to fake
 *   one. PDF export (`src/pdf.rs`) loads the same single Regular file into
 *   `usvg`'s `fontdb`, which never synthesizes either, so bold/italic are a
 *   no-op there too. Real bold/italic files don't currently exist upstream
 *   for either typeface.
 * - `monospace` renders in the preview with the bare CSS `monospace`
 *   keyword rather than a pinned custom font (see `textFontFamily` in
 *   PreviewSvgRenderer.tsx), so it resolves to the viewer's real system
 *   monospace font — which typically ships genuine bold/italic faces the
 *   browser can pick directly, no synthesis needed. (PDF export is the
 *   exception: it loads only `NotoSansMono-Regular.ttf`, so exported bold/
 *   italic monospace text is still a no-op there — a separate, currently
 *   undocumented preview/export mismatch.)
 */
const fontFamilyStyleCapabilities: Record<
  FontFamilyValue,
  { bold: boolean; italic: boolean }
> = {
  serif: { bold: false, italic: false },
  sans_serif: { bold: false, italic: false },
  monospace: { bold: true, italic: true },
}

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

const styleToggleButtonDisabledStyle: React.CSSProperties = {
  ...styleToggleButtonStyle,
  background: '#f5f5f5',
  borderColor: '#e2e8f0',
  color: '#bbb',
  cursor: 'not-allowed',
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
  disabledReason,
  ariaLabel,
  onClick,
}: {
  component: TextStyleBooleanComponent
  checked: boolean
  /** When set, the toggle renders disabled and this becomes its `title`
   * (e.g. explaining that the row's current font has no real bold/italic
   * face — see `fontFamilyStyleCapabilities`). */
  disabledReason?: string
  ariaLabel: string
  onClick: () => void
}) {
  const disabled = disabledReason != null
  return (
    <button
      type="button"
      aria-label={ariaLabel}
      aria-pressed={checked}
      disabled={disabled}
      title={disabledReason}
      style={
        disabled
          ? styleToggleButtonDisabledStyle
          : checked
            ? styleToggleButtonPressedStyle
            : styleToggleButtonStyle
      }
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
  // Rows without a font-family selector (notes/chords/note_dash) are always
  // measured in a fixed monospace font (see `StyleRowSpec.showFontFamily`'s
  // doc comment), which does support bold/italic — so only look up a
  // capability restriction when the row actually offers the selector.
  const effectiveFontFamily = showFontFamily
    ? (value.font_family ?? placeholder?.font_family ?? null)
    : null
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
        const unsupported =
          (component === 'bold' || component === 'italic') &&
          effectiveFontFamily != null &&
          !fontFamilyStyleCapabilities[effectiveFontFamily][component]
        const disabledReason = unsupported
          ? `${fontFamilyOptionLabels[effectiveFontFamily as FontFamilyValue]} has no ${component} face, so this has no visible effect`
          : undefined
        return (
          <td key={component} style={{ ...tdStyle, textAlign: 'center' }}>
            <StyleToggleButton
              component={component}
              checked={checked}
              disabledReason={disabledReason}
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
