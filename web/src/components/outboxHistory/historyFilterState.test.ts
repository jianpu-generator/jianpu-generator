import { describe, expect, it } from 'vitest'
import {
  EMPTY_FILTER_STATE,
  isFilterActive,
  toHistoryFilter,
} from './historyFilterState'

describe('toHistoryFilter', () => {
  it('maps an untouched form to a filter that matches everything', () => {
    expect(toHistoryFilter(EMPTY_FILTER_STATE)).toEqual({
      query: '',
      status: { tag: 'any' },
      kind: undefined,
      fileId: undefined,
      maxAgeMs: undefined,
      minSizeBytes: undefined,
      attempts: 'any',
    })
  })

  it('maps every control to its filter field', () => {
    expect(
      toHistoryFilter({
        query: 'canon',
        status: 'sent',
        kind: 'edit',
        fileId: 'f1',
        age: '24h',
        size: '3kb',
        tries: 'two-or-more',
      }),
    ).toEqual({
      query: 'canon',
      status: { tag: 'only', val: 'sent' },
      kind: 'edit',
      fileId: 'f1',
      maxAgeMs: 86_400_000n,
      minSizeBytes: 3072,
      attempts: 'two-or-more',
    })
  })

  it('maps "Any problem" to the problem status filter', () => {
    expect(
      toHistoryFilter({ ...EMPTY_FILTER_STATE, status: 'problem' }).status,
    ).toEqual({ tag: 'problem' })
  })
})

describe('isFilterActive', () => {
  it('is false for the empty state and true once any control is set', () => {
    expect(isFilterActive(EMPTY_FILTER_STATE)).toBe(false)
    expect(isFilterActive({ ...EMPTY_FILTER_STATE, size: '1kb' })).toBe(true)
  })
})
