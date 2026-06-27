export type MetadataKey =
  | 'title'
  | 'subtitle'
  | 'author'
  | 'row height'
  | 'max columns'
  | 'label width'
  | 'note number width'

export interface ParsedMetadataFields {
  title: string
  subtitle: string | null
  author: string | null
  rowHeight: number | null
  maxColumns: number | null
  labelWidth: number | null
  noteNumberWidth: number | null
}

const numericKeys: MetadataKey[] = [
  'row height',
  'max columns',
  'label width',
  'note number width',
]

const canonicalKeyOrder: MetadataKey[] = [
  'title',
  'subtitle',
  'author',
  'row height',
  'max columns',
  'label width',
  'note number width',
]

function isNumericKey(key: MetadataKey): boolean {
  return numericKeys.includes(key)
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
  return isNumericKey(key) ? `${key} = ${value}` : `${key} = "${value}"`
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
    rowHeight: null,
    maxColumns: null,
    labelWidth: null,
    noteNumberWidth: null,
  }

  if (startIndex === -1) return result

  const fieldMap = parseSectionIntoMap(lines, startIndex, endIndex)

  if (fieldMap.has('title')) result.title = fieldMap.get('title') as string
  if (fieldMap.has('subtitle'))
    result.subtitle = fieldMap.get('subtitle') as string
  if (fieldMap.has('author')) result.author = fieldMap.get('author') as string
  if (fieldMap.has('row height'))
    result.rowHeight = parseInt(fieldMap.get('row height') as string, 10)
  if (fieldMap.has('max columns'))
    result.maxColumns = parseInt(fieldMap.get('max columns') as string, 10)
  if (fieldMap.has('label width'))
    result.labelWidth = parseInt(fieldMap.get('label width') as string, 10)
  if (fieldMap.has('note number width'))
    result.noteNumberWidth = parseInt(
      fieldMap.get('note number width') as string,
      10,
    )

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
