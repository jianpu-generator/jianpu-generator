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

/** A `<kind> = { ... }` object's four components (see `syntax.md`). Any
 * subset may be set in the source; an unset component is `null` here. */
export const textStyleComponents = [
  'font_size',
  'horizontal_padding_pt',
  'vertical_padding_pt',
  'width_pt',
] as const

export type TextStyleComponent = (typeof textStyleComponents)[number]

export interface TextStyleFields {
  font_size: number | null
  horizontal_padding_pt: number | null
  vertical_padding_pt: number | null
  width_pt: number | null
}

function emptyStyle(): TextStyleFields {
  return {
    font_size: null,
    horizontal_padding_pt: null,
    vertical_padding_pt: null,
    width_pt: null,
  }
}

/** Scalar (non-text-style) `# metadata` keys — everything left over once
 * `title`/`subtitle`/`author`'s string content and the 13 `TextStyle` kinds
 * are accounted for. */
export type ScalarMetadataKey =
  | 'row_height'
  | 'max_measures_per_system'
  | 'note_number_width'
  | 'parts_list_columns'
  | 'merge_duplicate_measures_across_parts'
  | 'hide_resting_parts'
  | 'hide_system_dividers'
  | 'directive_row_offset'

const numericScalarKeys: ScalarMetadataKey[] = [
  'row_height',
  'max_measures_per_system',
  'note_number_width',
  'parts_list_columns',
]

/** Every editable `# metadata` field: `title`/`subtitle`/`author`'s string
 * content, a scalar key, or one `<kind>.<component>` pair addressing a
 * single component of a kind's `<kind> = { ... }` style object. */
export type MetadataFieldKey =
  | 'title'
  | 'subtitle'
  | 'author'
  | ScalarMetadataKey
  | `${TextStyleKind}.${TextStyleComponent}`

export interface ParsedMetadataFields {
  title: string
  subtitle: string | null
  author: string | null
  row_height: number | null
  max_measures_per_system: number | null
  note_number_width: number | null
  parts_list_columns: number | null
  merge_duplicate_measures_across_parts: boolean | null
  hide_resting_parts: boolean | null
  hide_system_dividers: boolean | null
  directive_row_offset: string | null
  styles: Record<TextStyleKind, TextStyleFields>
}

function emptyParsedMetadata(): ParsedMetadataFields {
  return {
    title: '',
    subtitle: null,
    author: null,
    row_height: null,
    max_measures_per_system: null,
    note_number_width: null,
    parts_list_columns: null,
    merge_duplicate_measures_across_parts: null,
    hide_resting_parts: null,
    hide_system_dividers: null,
    directive_row_offset: null,
    styles: Object.fromEntries(
      textStyleKinds.map((kind) => [kind, emptyStyle()]),
    ) as Record<TextStyleKind, TextStyleFields>,
  }
}

function isTextStyleKind(key: string): key is TextStyleKind {
  return (textStyleKinds as readonly string[]).includes(key)
}

function findMetadataSection(lines: string[]): {
  startIndex: number
  endIndex: number
} {
  const startIndex = lines.findIndex((line) => line.trim() === '# metadata')
  if (startIndex === -1) return { startIndex: -1, endIndex: -1 }

  let endIndex = lines.length
  for (let i = startIndex + 1; i < lines.length; i++) {
    if (lines[i].trimStart().startsWith('#')) {
      endIndex = i
      break
    }
  }

  return { startIndex, endIndex }
}

/** Parses a `<kind> = { font_size: N, ... }` object literal's inner text
 * (with or without the surrounding braces) into a `TextStyleFields` — any
 * component not present, or whose value doesn't parse as an integer, stays
 * `null`. */
function parseStyleObject(rawValue: string): TextStyleFields {
  const style = emptyStyle()
  const inner = rawValue.trim().replace(/^\{/, '').replace(/\}$/, '')
  for (const entry of inner.split(',')) {
    const colonIndex = entry.indexOf(':')
    if (colonIndex === -1) continue
    const component = entry.slice(0, colonIndex).trim()
    const value = Number.parseInt(entry.slice(colonIndex + 1).trim(), 10)
    if (Number.isNaN(value)) continue
    if ((textStyleComponents as readonly string[]).includes(component)) {
      style[component as TextStyleComponent] = value
    }
  }
  return style
}

export function parseMetadata(source: string): ParsedMetadataFields {
  const lines = source.split('\n')
  const { startIndex, endIndex } = findMetadataSection(lines)
  const result = emptyParsedMetadata()
  if (startIndex === -1) return result

  for (let i = startIndex + 1; i < endIndex; i++) {
    const line = lines[i]
    if (line.trim() === '') continue

    const eqIndex = line.indexOf('=')
    if (eqIndex === -1) continue

    const key = line.slice(0, eqIndex).trim()
    const rawValue = line.slice(eqIndex + 1).trim()

    if (rawValue.startsWith('{')) {
      if (isTextStyleKind(key)) {
        result.styles[key] = {
          ...result.styles[key],
          ...parseStyleObject(rawValue),
        }
      }
      continue
    }

    const value = rawValue.replace(/^"(.*)"$/, '$1')
    switch (key) {
      case 'title':
        result.title = value
        break
      case 'subtitle':
        result.subtitle = value
        break
      case 'author':
        result.author = value
        break
      case 'row_height':
        result.row_height = Number.parseInt(value, 10)
        break
      case 'max_measures_per_system':
        result.max_measures_per_system = Number.parseInt(value, 10)
        break
      case 'note_number_width':
        result.note_number_width = Number.parseInt(value, 10)
        break
      case 'parts_list_columns':
        result.parts_list_columns = Number.parseInt(value, 10)
        break
      case 'merge_duplicate_measures_across_parts':
        result.merge_duplicate_measures_across_parts = value === 'yes'
        break
      case 'hide_resting_parts':
        result.hide_resting_parts = value === 'yes'
        break
      case 'hide_system_dividers':
        result.hide_system_dividers = value === 'yes'
        break
      case 'directive_row_offset':
        result.directive_row_offset = value
        break
      default:
        break
    }
  }

  return result
}

function applyFieldUpdate(
  parsed: ParsedMetadataFields,
  key: MetadataFieldKey,
  value: string | null,
): void {
  if (key === 'title' || key === 'subtitle' || key === 'author') {
    const content = value === null ? null : value
    if (key === 'title') {
      parsed.title = content ?? ''
    } else {
      parsed[key] = content === '' ? null : content
    }
    return
  }

  const dotIndex = key.indexOf('.')
  if (dotIndex !== -1) {
    const kind = key.slice(0, dotIndex) as TextStyleKind
    const component = key.slice(dotIndex + 1) as TextStyleComponent
    const parsedNum =
      value === null || value === '' ? null : Number.parseInt(value, 10)
    parsed.styles[kind] = {
      ...parsed.styles[kind],
      [component]:
        parsedNum === null || Number.isNaN(parsedNum) ? null : parsedNum,
    }
    return
  }

  const scalarKey = key as ScalarMetadataKey
  if (value === null || value === '') {
    // Checkboxes always write an explicit yes/no (see `setYesNo` in
    // `EditMetadataModal`), so a null/empty value here only clears
    // `directive_row_offset` or one of the numeric scalars.
    if (scalarKey === 'directive_row_offset') {
      parsed.directive_row_offset = null
    } else if (numericScalarKeys.includes(scalarKey)) {
      ;(parsed[scalarKey] as number | null) = null
    }
    return
  }

  switch (scalarKey) {
    case 'row_height':
      parsed.row_height = Number.parseInt(value, 10)
      break
    case 'max_measures_per_system':
      parsed.max_measures_per_system = Number.parseInt(value, 10)
      break
    case 'note_number_width':
      parsed.note_number_width = Number.parseInt(value, 10)
      break
    case 'parts_list_columns':
      parsed.parts_list_columns = Number.parseInt(value, 10)
      break
    case 'merge_duplicate_measures_across_parts':
      parsed.merge_duplicate_measures_across_parts = value === 'yes'
      break
    case 'hide_resting_parts':
      parsed.hide_resting_parts = value === 'yes'
      break
    case 'hide_system_dividers':
      parsed.hide_system_dividers = value === 'yes'
      break
    case 'directive_row_offset':
      parsed.directive_row_offset = value
      break
    default:
      break
  }
}

function formatStyleValue(style: TextStyleFields): string | null {
  const parts = textStyleComponents
    .filter((component) => style[component] !== null)
    .map((component) => `${component}: ${style[component]}`)
  return parts.length === 0 ? null : `{ ${parts.join(', ')} }`
}

function emitCanonicalSection(parsed: ParsedMetadataFields): string[] {
  const lines: string[] = []

  const titleStyle = formatStyleValue(parsed.styles.title)
  if (parsed.title !== '') lines.push(`title = "${parsed.title}"`)
  if (titleStyle) lines.push(`title = ${titleStyle}`)

  const subtitleStyle = formatStyleValue(parsed.styles.subtitle)
  if (parsed.subtitle !== null) lines.push(`subtitle = "${parsed.subtitle}"`)
  if (subtitleStyle) lines.push(`subtitle = ${subtitleStyle}`)

  const authorStyle = formatStyleValue(parsed.styles.author)
  if (parsed.author !== null) lines.push(`author = "${parsed.author}"`)
  if (authorStyle) lines.push(`author = ${authorStyle}`)

  if (parsed.row_height !== null)
    lines.push(`row_height = ${parsed.row_height}`)
  if (parsed.max_measures_per_system !== null)
    lines.push(`max_measures_per_system = ${parsed.max_measures_per_system}`)
  if (parsed.note_number_width !== null)
    lines.push(`note_number_width = ${parsed.note_number_width}`)
  if (parsed.parts_list_columns !== null)
    lines.push(`parts_list_columns = ${parsed.parts_list_columns}`)

  for (const kind of textStyleKinds) {
    if (kind === 'title' || kind === 'subtitle' || kind === 'author') continue
    const value = formatStyleValue(parsed.styles[kind])
    if (value) lines.push(`${kind} = ${value}`)
  }

  if (parsed.merge_duplicate_measures_across_parts !== null)
    lines.push(
      `merge_duplicate_measures_across_parts = ${
        parsed.merge_duplicate_measures_across_parts ? 'yes' : 'no'
      }`,
    )
  if (parsed.hide_resting_parts !== null)
    lines.push(
      `hide_resting_parts = ${parsed.hide_resting_parts ? 'yes' : 'no'}`,
    )
  if (parsed.hide_system_dividers !== null)
    lines.push(
      `hide_system_dividers = ${parsed.hide_system_dividers ? 'yes' : 'no'}`,
    )
  if (parsed.directive_row_offset !== null)
    lines.push(`directive_row_offset = ${parsed.directive_row_offset}`)

  lines.push('')
  return lines
}

export function updateMetadataField(
  source: string,
  key: MetadataFieldKey,
  value: string | null,
): string {
  const lines = source.split('\n')
  const { startIndex, endIndex } = findMetadataSection(lines)
  if (startIndex === -1) return source

  const parsed = parseMetadata(source)
  applyFieldUpdate(parsed, key, value)

  const canonicalLines = emitCanonicalSection(parsed)

  const updated = [
    ...lines.slice(0, startIndex + 1),
    ...canonicalLines,
    ...lines.slice(endIndex),
  ]

  return updated.join('\n')
}
