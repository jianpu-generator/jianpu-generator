export type {
  FontFamilyValue,
  TextStyleBooleanComponent,
  TextStyleComponent,
  TextStyleEnumComponent,
  TextStyleFields,
  TextStyleKind,
  TextStyleNumericComponent,
} from './textStyleFields'
export {
  emptyStyle,
  fontFamilyValues,
  isTextStyleBooleanComponent,
  isTextStyleEnumComponent,
  textStyleBooleanComponents,
  textStyleComponents,
  textStyleEnumComponents,
  textStyleKinds,
  textStyleNumericComponents,
} from './textStyleFields'

import type {
  FontFamilyValue,
  TextStyleBooleanComponent,
  TextStyleComponent,
  TextStyleFields,
  TextStyleKind,
  TextStyleNumericComponent,
} from './textStyleFields'
import {
  emptyStyle,
  fontFamilyValues,
  isTextStyleBooleanComponent,
  isTextStyleEnumComponent,
  textStyleComponents,
  textStyleKinds,
} from './textStyleFields'

/** Scalar (non-text-style) `# metadata` keys — everything left over once
 * `title`/`subtitle`/`author`'s string content and the 13 `TextStyle` kinds
 * are accounted for. `part_label_width_pt` is a flat scalar (a layout
 * constant, not a text style component) rather than `part_label.width_pt`. */
export type ScalarMetadataKey =
  | 'row_height'
  | 'max_measures_per_system'
  | 'note_number_width'
  | 'parts_list_columns'
  | 'part_label_width_pt'
  | 'merge_duplicate_measures_across_parts'
  | 'hide_resting_parts'
  | 'hide_system_dividers'
  | 'directive_row_offset'

export const numericScalarKeys: ScalarMetadataKey[] = [
  'row_height',
  'max_measures_per_system',
  'note_number_width',
  'parts_list_columns',
  'part_label_width_pt',
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
  part_label_width_pt: number | null
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
    part_label_width_pt: null,
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

export function findMetadataSection(lines: string[]): {
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
    if (!(textStyleComponents as readonly string[]).includes(component)) {
      continue
    }
    const rawComponentValue = entry.slice(colonIndex + 1).trim()
    if (isTextStyleBooleanComponent(component as TextStyleComponent)) {
      if (rawComponentValue !== 'yes' && rawComponentValue !== 'no') continue
      style[component as TextStyleBooleanComponent] =
        rawComponentValue === 'yes'
    } else if (isTextStyleEnumComponent(component as TextStyleComponent)) {
      if (!(fontFamilyValues as readonly string[]).includes(rawComponentValue))
        continue
      style.font_family = rawComponentValue as FontFamilyValue
    } else {
      const value = Number.parseInt(rawComponentValue, 10)
      if (Number.isNaN(value)) continue
      style[component as TextStyleNumericComponent] = value
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
      case 'part_label_width_pt':
        result.part_label_width_pt = Number.parseInt(value, 10)
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

export { updateMetadataField } from './metadataSourceWrite'
