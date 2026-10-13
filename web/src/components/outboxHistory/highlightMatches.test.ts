import { describe, expect, it } from 'vitest'
import { splitByQuery } from './highlightMatches'

describe('splitByQuery', () => {
  it('returns one unmatched run for an empty query', () => {
    expect(splitByQuery('Ode to Joy', '')).toEqual([
      { text: 'Ode to Joy', matched: false },
    ])
  })

  it('marks every case-insensitive occurrence and keeps the original casing', () => {
    expect(splitByQuery('Canon in D, canon again', 'CANON')).toEqual([
      { text: 'Canon', matched: true },
      { text: ' in D, ', matched: false },
      { text: 'canon', matched: true },
      { text: ' again', matched: false },
    ])
  })

  it('returns the text unmatched when nothing is found', () => {
    expect(splitByQuery('abc', 'z')).toEqual([{ text: 'abc', matched: false }])
  })
})
