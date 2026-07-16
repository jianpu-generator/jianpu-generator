export type MetadataKey =
  | 'title'
  | 'subtitle'
  | 'author'
  | 'row height'
  | 'max measures per system'
  | 'label width'
  | 'note number width'
  | 'parts list columns'
  | 'lyrics font size'
  | 'merge duplicate measures across parts'
  | 'hide resting parts'
  | 'hide system dividers'

export interface ParsedMetadataFields {
  title: string
  subtitle: string | null
  author: string | null
  rowHeight: number | null
  maxMeasuresPerSystem: number | null
  labelWidth: number | null
  noteNumberWidth: number | null
  partsListColumns: number | null
  lyricsFontSize: number | null
  mergeDuplicateMeasuresAcrossParts: boolean | null
  hideRestingParts: boolean | null
  hideSystemDividers: boolean | null
}

const numericKeys: MetadataKey[] = [
  'row height',
  'max measures per system',
  'label width',
  'note number width',
  'parts list columns',
  'lyrics font size',
]

const unquotedKeys: MetadataKey[] = [
  ...numericKeys,
  'merge duplicate measures across parts',
  'hide resting parts',
  'hide system dividers',
]

const canonicalKeyOrder: MetadataKey[] = [
  'title',
  'subtitle',
  'author',
  'row height',
  'max measures per system',
  'label width',
  'note number width',
  'parts list columns',
  'lyrics font size',
  'merge duplicate measures across parts',
  'hide resting parts',
  'hide system dividers',
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
    rowHeight: null,
    maxMeasuresPerSystem: null,
    labelWidth: null,
    noteNumberWidth: null,
    partsListColumns: null,
    lyricsFontSize: null,
    mergeDuplicateMeasuresAcrossParts: null,
    hideRestingParts: null,
    hideSystemDividers: null,
  }

  if (startIndex === -1) return result

  const fieldMap = parseSectionIntoMap(lines, startIndex, endIndex)

  if (fieldMap.has('title')) result.title = fieldMap.get('title') as string
  if (fieldMap.has('subtitle'))
    result.subtitle = fieldMap.get('subtitle') as string
  if (fieldMap.has('author')) result.author = fieldMap.get('author') as string
  if (fieldMap.has('row height'))
    result.rowHeight = parseInt(fieldMap.get('row height') as string, 10)
  if (fieldMap.has('max measures per system'))
    result.maxMeasuresPerSystem = parseInt(
      fieldMap.get('max measures per system') as string,
      10,
    )
  if (fieldMap.has('label width'))
    result.labelWidth = parseInt(fieldMap.get('label width') as string, 10)
  if (fieldMap.has('note number width'))
    result.noteNumberWidth = parseInt(
      fieldMap.get('note number width') as string,
      10,
    )
  if (fieldMap.has('parts list columns'))
    result.partsListColumns = parseInt(
      fieldMap.get('parts list columns') as string,
      10,
    )
  if (fieldMap.has('lyrics font size'))
    result.lyricsFontSize = parseInt(
      fieldMap.get('lyrics font size') as string,
      10,
    )
  if (fieldMap.has('merge duplicate measures across parts'))
    result.mergeDuplicateMeasuresAcrossParts =
      fieldMap.get('merge duplicate measures across parts') === 'yes'
  if (fieldMap.has('hide resting parts'))
    result.hideRestingParts = fieldMap.get('hide resting parts') === 'yes'
  if (fieldMap.has('hide system dividers'))
    result.hideSystemDividers = fieldMap.get('hide system dividers') === 'yes'

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
