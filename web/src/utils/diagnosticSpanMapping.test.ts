import type { PartMeasureRangesOut } from 'jianpu-wasm'
import { describe, expect, it } from 'vitest'
import type { MeasureSpan } from '../types'
import { mapZippedSpanToUnzippedRange } from './diagnosticSpanMapping'

function measureSpan(start: number, end: number): MeasureSpan {
  return {
    start,
    end,
    view_zone_start: start,
    start_line: 1,
    end_line: 1,
  }
}

describe('mapZippedSpanToUnzippedRange', () => {
  const measureSpans: MeasureSpan[] = [measureSpan(0, 10), measureSpan(10, 20)]
  const partMeasureRanges: PartMeasureRangesOut[] = [
    {
      abbreviation: 'S',
      ranges: [
        { start: 0, end: 5 },
        { start: 5, end: 12 },
      ],
    },
    {
      abbreviation: 'A',
      ranges: [
        { start: 20, end: 25 },
        { start: 25, end: 30 },
      ],
    },
  ]

  it('maps a span inside the first measure to the first part range', () => {
    expect(
      mapZippedSpanToUnzippedRange(
        { start: 2, end: 3 },
        measureSpans,
        partMeasureRanges,
      ),
    ).toEqual({ start: 0, end: 5 })
  })

  it('maps a span inside the second measure to the first part range', () => {
    expect(
      mapZippedSpanToUnzippedRange(
        { start: 15, end: 16 },
        measureSpans,
        partMeasureRanges,
      ),
    ).toEqual({ start: 5, end: 12 })
  })

  it('returns null when the span falls outside every measure', () => {
    expect(
      mapZippedSpanToUnzippedRange(
        { start: 25, end: 26 },
        measureSpans,
        partMeasureRanges,
      ),
    ).toBeNull()
  })

  it('returns null when no measure spans are known', () => {
    expect(
      mapZippedSpanToUnzippedRange({ start: 2, end: 3 }, [], partMeasureRanges),
    ).toBeNull()
  })

  it('returns null when no part covers the matched measure', () => {
    expect(
      mapZippedSpanToUnzippedRange({ start: 2, end: 3 }, measureSpans, []),
    ).toBeNull()
  })
})
