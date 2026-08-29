/** Every rendered text kind, matching `Metadata`'s per-kind `TextStyle`
 * fields on the Rust side (see `syntax.md`'s "Text styles" section) — the
 * order here is also the canonical emission order for their `<kind> = {...}`
 * lines. */
export const textStyleKinds = [
  'title',
  'subtitle',
  'author',
  'sequence',
  'part_legend',
  'measure_number',
  'section_label',
  'part_label',
  'page_number',
  'lyrics',
  'notes',
  'chords',
  'note_dash',
] as const

export type TextStyleKind = (typeof textStyleKinds)[number]

/** A `<kind> = { ... }` object's numeric components (see `syntax.md`). Any
 * subset may be set in the source; an unset component is `null` here. */
export const textStyleNumericComponents = [
  'font_size',
  'horizontal_padding_pt',
  'vertical_padding_pt',
] as const

export type TextStyleNumericComponent =
  (typeof textStyleNumericComponents)[number]

/** A `<kind> = { ... }` object's boolean (`yes`/`no`) components (see
 * `syntax.md`). */
export const textStyleBooleanComponents = [
  'bold',
  'italic',
  'underline',
] as const

export type TextStyleBooleanComponent =
  (typeof textStyleBooleanComponents)[number]

export const textStyleComponents = [
  ...textStyleNumericComponents,
  ...textStyleBooleanComponents,
] as const

export type TextStyleComponent = (typeof textStyleComponents)[number]

export function isTextStyleBooleanComponent(
  component: TextStyleComponent,
): component is TextStyleBooleanComponent {
  return (textStyleBooleanComponents as readonly string[]).includes(component)
}

export interface TextStyleFields {
  font_size: number | null
  horizontal_padding_pt: number | null
  vertical_padding_pt: number | null
  bold: boolean | null
  italic: boolean | null
  underline: boolean | null
}

export function emptyStyle(): TextStyleFields {
  return {
    font_size: null,
    horizontal_padding_pt: null,
    vertical_padding_pt: null,
    bold: null,
    italic: null,
    underline: null,
  }
}
