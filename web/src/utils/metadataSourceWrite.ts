/** Write side of metadata-source editing: applying a single field edit to a
 * `ParsedMetadataFields` and re-serializing the whole `# metadata` section
 * back into canonical `.jianpu` source text. Split out from
 * `metadataSource.ts` (the read/parse side) to keep that file under the
 * repo's max-file-lines limit. */
import type {
  FontFamilyValue,
  MetadataFieldKey,
  ParsedMetadataFields,
  ScalarMetadataKey,
} from './metadataSource'
import {
  findMetadataSection,
  numericScalarKeys,
  parseMetadata,
} from './metadataSource'
import type {
  TextStyleComponent,
  TextStyleFields,
  TextStyleKind,
} from './textStyleFields'
import {
  fontFamilyValues,
  isTextStyleBooleanComponent,
  isTextStyleEnumComponent,
  textStyleComponents,
  textStyleKinds,
} from './textStyleFields'

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
    const componentValue = isTextStyleBooleanComponent(component)
      ? value === null || value === ''
        ? null
        : value === 'yes'
      : isTextStyleEnumComponent(component)
        ? value === null || value === ''
          ? null
          : (fontFamilyValues as readonly string[]).includes(value)
            ? (value as FontFamilyValue)
            : null
        : (() => {
            const parsedNum =
              value === null || value === '' ? null : Number.parseInt(value, 10)
            return parsedNum === null || Number.isNaN(parsedNum)
              ? null
              : parsedNum
          })()
    parsed.styles[kind] = {
      ...parsed.styles[kind],
      [component]: componentValue,
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
    case 'part_label_width_pt':
      parsed.part_label_width_pt = Number.parseInt(value, 10)
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
    .map((component) => {
      const value = style[component]
      const formatted = isTextStyleBooleanComponent(component)
        ? value
          ? 'yes'
          : 'no'
        : value
      return `${component}: ${formatted}`
    })
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
  if (parsed.part_label_width_pt !== null)
    lines.push(`part_label_width_pt = ${parsed.part_label_width_pt}`)

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
