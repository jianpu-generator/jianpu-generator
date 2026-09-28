import { describe, expect, it } from 'vitest'
import { distinctPartAbbreviations } from './partAbbreviations'

describe('distinctPartAbbreviations', () => {
  it('dedupes abbreviations in first-seen order', () => {
    expect(
      distinctPartAbbreviations([
        { partAbbreviation: 'B' },
        { partAbbreviation: 'S' },
        { partAbbreviation: 'B' },
      ]),
    ).toEqual(['B', 'S'])
  })

  it('skips items without an abbreviation', () => {
    expect(
      distinctPartAbbreviations([{}, { partAbbreviation: 'S' }, {}]),
    ).toEqual(['S'])
  })
})
