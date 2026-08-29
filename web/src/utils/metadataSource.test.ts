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
row_height = 80
max_measures_per_system = 4
note_number_width = 10
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
row_height = 80
subtitle = "Sub"
# notes
1 2 3`
    const result = updateMetadataField(
      outOfOrderSource,
      'max_measures_per_system',
      '4',
    )
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
      'row_height',
      'max_measures_per_system',
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
    expect(result).not.toContain('row_height')
    expect(result).not.toContain('max_measures_per_system')
    expect(result).not.toContain('note_number_width')
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
    const result = updateMetadataField(sourceWithMetadata, 'row_height', '100')
    expect(result).toContain('row_height = 100')
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

  it('formats the merge_duplicate_measures_across_parts key without quotes', () => {
    const result = updateMetadataField(
      sourceWithMetadata,
      'merge_duplicate_measures_across_parts',
      'no',
    )
    expect(result).toContain('merge_duplicate_measures_across_parts = no')
    expect(result).not.toContain('"no"')
  })

  it('formats the hide_system_dividers key without quotes', () => {
    const result = updateMetadataField(
      sourceWithMetadata,
      'hide_system_dividers',
      'yes',
    )
    expect(result).toContain('hide_system_dividers = yes')
    expect(result).not.toContain('"yes"')
  })

  it('formats string keys with quotes', () => {
    const result = updateMetadataField(
      sourceWithMetadata,
      'title',
      'Hello World',
    )
    expect(result).toContain('title = "Hello World"')
  })

  it('formats a text-style bold/italic/underline component as yes/no', () => {
    const result = updateMetadataField(sourceWithMetadata, 'title.bold', 'yes')
    expect(result).toContain('title = { bold: yes }')
  })

  it('round-trips a text-style boolean component alongside numeric ones', () => {
    const withFontSize = updateMetadataField(
      sourceWithMetadata,
      'title.font_size',
      '32',
    )
    const result = updateMetadataField(withFontSize, 'title.italic', 'yes')
    expect(result).toContain('title = { font_size: 32, italic: yes }')
  })

  it('clears a text-style boolean component back to unset', () => {
    const withBold = updateMetadataField(
      sourceWithMetadata,
      'title.bold',
      'yes',
    )
    const result = updateMetadataField(withBold, 'title.bold', '')
    expect(result).not.toContain('bold')
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
      'row_height',
      'max_measures_per_system',
      'note_number_width',
    ])
  })
})
