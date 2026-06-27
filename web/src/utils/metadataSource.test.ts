import { describe, expect, it } from 'vitest'
import { updateMetadataField } from './metadataSource'

const sourceWithMetadata = `# metadata
title = "My Song"
author = "Alice"

# notes
1 2 3 4`

const sourceWithAllFields = `# metadata
title = "Song"
subtitle = "Sub"
author = "Bob"
row height = 80
max columns = 4
label width = 20
note number width = 10
# notes
1 2 3`

describe('updateMetadataField', () => {
  it('adds a new field to a source that already has some metadata', () => {
    const result = updateMetadataField(
      sourceWithMetadata,
      'subtitle',
      'A Subtitle',
    )
    expect(result).toContain('subtitle = "A Subtitle"')
  })

  it('updates an existing field', () => {
    const result = updateMetadataField(sourceWithMetadata, 'title', 'New Title')
    expect(result).toContain('title = "New Title"')
    expect(result).not.toContain('title = "My Song"')
  })

  it('removes a field when value is null', () => {
    const result = updateMetadataField(sourceWithMetadata, 'author', null)
    expect(result).not.toContain('author')
  })

  it('removes a field when value is empty string', () => {
    const result = updateMetadataField(sourceWithMetadata, 'author', '')
    expect(result).not.toContain('author')
  })

  it('strips blank lines in the original metadata section (only trailing blank line remains)', () => {
    const result = updateMetadataField(sourceWithMetadata, 'title', 'Fixed')
    const allLines = result.split('\n')
    const metadataStart = allLines.findIndex((l) => l.trim() === '# metadata')
    const metadataEnd = allLines.findIndex(
      (l, i) => i > metadataStart && l.trimStart().startsWith('#'),
    )
    const bodyLines = allLines.slice(metadataStart + 1, metadataEnd)
    const nonTrailingLines = bodyLines.slice(0, -1)
    expect(nonTrailingLines.every((line) => line !== '')).toBe(true)
    expect(bodyLines.at(-1)).toBe('')
  })

  it('emits fields in canonical order regardless of input order', () => {
    const outOfOrderSource = `# metadata
author = "Bob"
title = "Song"
row height = 80
subtitle = "Sub"
# notes
1 2 3`
    const result = updateMetadataField(outOfOrderSource, 'max columns', '4')
    const lines = result.split('\n')
    const metadataStart = lines.findIndex((l) => l.trim() === '# metadata')
    const metadataEnd = lines.findIndex(
      (l, i) => i > metadataStart && l.trimStart().startsWith('#'),
    )
    const metadataLines = lines
      .slice(metadataStart + 1, metadataEnd)
      .map((l) => l.split('=')[0].trim())
      .filter((k) => k !== '')

    expect(metadataLines).toEqual([
      'title',
      'subtitle',
      'author',
      'row height',
      'max columns',
    ])
  })

  it('does not emit lines for fields that are not set', () => {
    const minimalSource = `# metadata
title = "Solo"
# notes
1`
    const result = updateMetadataField(minimalSource, 'title', 'Solo Updated')
    expect(result).not.toContain('subtitle')
    expect(result).not.toContain('author')
    expect(result).not.toContain('row height')
    expect(result).not.toContain('max columns')
    expect(result).not.toContain('label width')
    expect(result).not.toContain('note number width')
  })

  it('leaves sections before metadata untouched', () => {
    const sourceWithPreamble = `# header
some preamble text
# metadata
title = "Song"
# notes
1 2`
    const result = updateMetadataField(sourceWithPreamble, 'title', 'Changed')
    expect(result.startsWith('# header\nsome preamble text\n')).toBe(true)
  })

  it('leaves sections after metadata untouched', () => {
    const result = updateMetadataField(sourceWithMetadata, 'title', 'Changed')
    expect(result).toContain('# notes\n1 2 3 4')
  })

  it('formats numeric keys without quotes', () => {
    const result = updateMetadataField(sourceWithMetadata, 'row height', '100')
    expect(result).toContain('row height = 100')
    expect(result).not.toContain('"100"')
  })

  it('emits a trailing blank line after the metadata section', () => {
    const result = updateMetadataField(sourceWithMetadata, 'title', 'Fixed')
    const lines = result.split('\n')
    const metadataStart = lines.findIndex((l) => l.trim() === '# metadata')
    const metadataEnd = lines.findIndex(
      (l, i) => i > metadataStart && l.trimStart().startsWith('#'),
    )
    expect(lines[metadataEnd - 1]).toBe('')
  })

  it('formats string keys with quotes', () => {
    const result = updateMetadataField(
      sourceWithMetadata,
      'title',
      'Hello World',
    )
    expect(result).toContain('title = "Hello World"')
  })

  it('handles all fields present and sorted correctly', () => {
    const result = updateMetadataField(sourceWithAllFields, 'title', 'Updated')
    const lines = result.split('\n')
    const metadataStart = lines.findIndex((l) => l.trim() === '# metadata')
    const metadataEnd = lines.findIndex(
      (l, i) => i > metadataStart && l.trimStart().startsWith('#'),
    )
    const keys = lines
      .slice(metadataStart + 1, metadataEnd)
      .map((l) => l.split('=')[0].trim())
      .filter((k) => k !== '')

    expect(keys).toEqual([
      'title',
      'subtitle',
      'author',
      'row height',
      'max columns',
      'label width',
      'note number width',
    ])
  })
})
