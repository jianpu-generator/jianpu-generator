export type MetadataKey =
  | 'title'
  | 'subtitle'
  | 'author'
  | 'row_height'
  | 'max_measures_per_system'
  | 'note_number_width'
  | 'part_label_width_pt'
  | 'parts_list_columns'
  | 'lyrics_font_size'
  | 'merge_duplicate_measures_across_parts'
  | 'hide_resting_parts'
  | 'hide_system_dividers'
  | 'section_label_offset'

export interface ParsedMetadataFields {
  title: string
  subtitle: string | null
  author: string | null
  row_height: number | null
  max_measures_per_system: number | null
  note_number_width: number | null
  part_label_width_pt: number | null
  parts_list_columns: number | null
  lyrics_font_size: number | null
  merge_duplicate_measures_across_parts: boolean | null
  hide_resting_parts: boolean | null
  hide_system_dividers: boolean | null
  section_label_offset: string | null
}

const numericKeys: MetadataKey[] = [
  'row_height',
  'max_measures_per_system',
  'note_number_width',
  'part_label_width_pt',
  'parts_list_columns',
  'lyrics_font_size',
]

const unquotedKeys: MetadataKey[] = [
  ...numericKeys,
  'merge_duplicate_measures_across_parts',
  'hide_resting_parts',
  'hide_system_dividers',
  'section_label_offset',
]

const canonicalKeyOrder: MetadataKey[] = [
  'title',
  'subtitle',
  'author',
  'row_height',
  'max_measures_per_system',
  'note_number_width',
  'part_label_width_pt',
  'parts_list_columns',
  'lyrics_font_size',
  'merge_duplicate_measures_across_parts',
  'hide_resting_parts',
  'hide_system_dividers',
  'section_label_offset',
]

function isUnquotedKey(key: MetadataKey): boolean {
  return unquotedKeys.includes(key)
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

function parseSectionIntoMap(
  lines: string[],
  startIndex: number,
  endIndex: number,
): Map<MetadataKey, string> {
  const map = new Map<MetadataKey, string>()

  for (let i = startIndex + 1; i < endIndex; i++) {
    const line = lines[i]
    const trimmed = line.trim()
    if (trimmed === '') continue

    const eqIndex = line.indexOf('=')
    if (eqIndex === -1) continue

    const key = line.slice(0, eqIndex).trim() as MetadataKey
    const rawValue = line.slice(eqIndex + 1).trim()
    const value = rawValue.replace(/^"(.*)"$/, '$1')

    map.set(key, value)
  }

  return map
}

function formatMetadataLine(key: MetadataKey, value: string): string {
  return isUnquotedKey(key) ? `${key} = ${value}` : `${key} = "${value}"`
}

function emitCanonicalSection(fieldMap: Map<MetadataKey, string>): string[] {
  return [
    ...canonicalKeyOrder
      .filter((key) => fieldMap.has(key))
      .map((key) => formatMetadataLine(key, fieldMap.get(key) as string)),
    '',
  ]
}

export function parseMetadata(source: string): ParsedMetadataFields {
  const lines = source.split('\n')
  const { startIndex, endIndex } = findMetadataSection(lines)

  const result: ParsedMetadataFields = {
    title: '',
    subtitle: null,
    author: null,
    row_height: null,
    max_measures_per_system: null,
    note_number_width: null,
    part_label_width_pt: null,
    parts_list_columns: null,
    lyrics_font_size: null,
    merge_duplicate_measures_across_parts: null,
    hide_resting_parts: null,
    hide_system_dividers: null,
    section_label_offset: null,
  }

  if (startIndex === -1) return result

  const fieldMap = parseSectionIntoMap(lines, startIndex, endIndex)

  if (fieldMap.has('title')) result.title = fieldMap.get('title') as string
  if (fieldMap.has('subtitle'))
    result.subtitle = fieldMap.get('subtitle') as string
  if (fieldMap.has('author')) result.author = fieldMap.get('author') as string
  if (fieldMap.has('row_height'))
    result.row_height = parseInt(fieldMap.get('row_height') as string, 10)
  if (fieldMap.has('max_measures_per_system'))
    result.max_measures_per_system = parseInt(
      fieldMap.get('max_measures_per_system') as string,
      10,
    )
  if (fieldMap.has('note_number_width'))
    result.note_number_width = parseInt(
      fieldMap.get('note_number_width') as string,
      10,
    )
  if (fieldMap.has('part_label_width_pt'))
    result.part_label_width_pt = parseInt(
      fieldMap.get('part_label_width_pt') as string,
      10,
    )
  if (fieldMap.has('parts_list_columns'))
    result.parts_list_columns = parseInt(
      fieldMap.get('parts_list_columns') as string,
      10,
    )
  if (fieldMap.has('lyrics_font_size'))
    result.lyrics_font_size = parseInt(
      fieldMap.get('lyrics_font_size') as string,
      10,
    )
  if (fieldMap.has('merge_duplicate_measures_across_parts'))
    result.merge_duplicate_measures_across_parts =
      fieldMap.get('merge_duplicate_measures_across_parts') === 'yes'
  if (fieldMap.has('hide_resting_parts'))
    result.hide_resting_parts = fieldMap.get('hide_resting_parts') === 'yes'
  if (fieldMap.has('hide_system_dividers'))
    result.hide_system_dividers = fieldMap.get('hide_system_dividers') === 'yes'
  if (fieldMap.has('section_label_offset'))
    result.section_label_offset = fieldMap.get('section_label_offset') as string

  return result
}

export function updateMetadataField(
  source: string,
  key: MetadataKey,
  value: string | null,
): string {
  const lines = source.split('\n')
  const { startIndex, endIndex } = findMetadataSection(lines)
  if (startIndex === -1) return source

  const fieldMap = parseSectionIntoMap(lines, startIndex, endIndex)

  if (value === null || value === '') {
    fieldMap.delete(key)
  } else {
    fieldMap.set(key, value)
  }

  const canonicalLines = emitCanonicalSection(fieldMap)

  const updated = [
    ...lines.slice(0, startIndex + 1),
    ...canonicalLines,
    ...lines.slice(endIndex),
  ]

  return updated.join('\n')
}
