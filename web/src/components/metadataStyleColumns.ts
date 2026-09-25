/** The Text Styles table's column definitions, split out of
 * `MetadataStylesTable.tsx` to keep it under the max-file-lines cap. Each
 * column names its `TextStyleFields` key and builds its own
 * `TextStyleComponentValue` edit, both checked against the generated wasm
 * types. */
import type {
  FontFamilyChoice,
  TextStyleComponentValue,
  TextStyleFields,
} from '../jianpuWasm'

/** `TextStyleFields`' keys whose value is a number. */
type NumericField = {
  [Field in keyof TextStyleFields]-?: TextStyleFields[Field] extends
    | number
    | undefined
    ? Field
    : never
}[keyof TextStyleFields]

/** `TextStyleFields`' keys whose value is a `yes`/`no` flag. */
type BooleanField = {
  [Field in keyof TextStyleFields]-?: TextStyleFields[Field] extends
    | boolean
    | undefined
    ? Field
    : never
}[keyof TextStyleFields]

/** One numeric column: which field it shows, its header (also used to
 * build each input's `aria-label` as `"${rowLabel} ${subLabel}"`, so e2e
 * tests can target one component by accessible name rather than positional
 * `nth()` indexing), and the edit that sets it. */
interface NumericColumn {
  field: NumericField
  subLabel: string
  edit: (val: number | undefined) => TextStyleComponentValue
}

export const numericColumns: NumericColumn[] = [
  {
    field: 'fontSize',
    subLabel: 'Font Size',
    edit: (val) => ({ tag: 'font-size', val }),
  },
  {
    field: 'horizontalPaddingPt',
    subLabel: 'H. Padding',
    edit: (val) => ({ tag: 'horizontal-padding-pt', val }),
  },
  {
    field: 'verticalPaddingPt',
    subLabel: 'V. Padding',
    edit: (val) => ({ tag: 'vertical-padding-pt', val }),
  },
]

/** One boolean toggle column: its full label (the button's `aria-label`,
 * as `"${rowLabel} ${label}"`), the single-letter glyph shown on the
 * button (styled to preview the effect it toggles), and the edit that sets
 * it. */
export interface BooleanColumn {
  field: BooleanField
  label: string
  glyph: string
  glyphStyle: React.CSSProperties
  edit: (val: boolean) => TextStyleComponentValue
}

export const booleanColumns: BooleanColumn[] = [
  {
    field: 'bold',
    label: 'Bold',
    glyph: 'B',
    glyphStyle: { fontWeight: 'bold' },
    edit: (val) => ({ tag: 'bold', val }),
  },
  {
    field: 'italic',
    label: 'Italic',
    glyph: 'I',
    glyphStyle: { fontStyle: 'italic' },
    edit: (val) => ({ tag: 'italic', val }),
  },
  {
    field: 'underline',
    label: 'Underline',
    glyph: 'U',
    glyphStyle: { textDecoration: 'underline' },
    edit: (val) => ({ tag: 'underline', val }),
  },
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
export const fontFamilyStyleCapabilities: Record<
  FontFamilyChoice,
  Partial<Record<BooleanField, boolean>>
> = {
  serif: { bold: false, italic: false },
  'sans-serif': { bold: false, italic: false },
  monospace: { bold: true, italic: true },
}

/** Display label for each `font_family` option, shown in the `<select>`
 * (see `StyleRowSpec.showFontFamily`). */
export const fontFamilyOptionLabels: Record<FontFamilyChoice, string> = {
  serif: 'Serif',
  'sans-serif': 'Sans Serif',
  monospace: 'Monospace',
}

export const fontFamilyChoices = Object.keys(
  fontFamilyOptionLabels,
) as FontFamilyChoice[]
