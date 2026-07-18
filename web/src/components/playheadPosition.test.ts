import { describe, expect, it } from 'vitest'
import {
  computePlayheadRect,
  findMeasureSegmentAtTime,
  resolvePlayheadSegment,
} from './playheadPosition'

describe('findMeasureSegmentAtTime', () => {
  const times = [0, 1, 2, 3]

  it('returns the segment containing t', () => {
    expect(findMeasureSegmentAtTime(times, 0)).toBe(0)
    expect(findMeasureSegmentAtTime(times, 0.5)).toBe(0)
    expect(findMeasureSegmentAtTime(times, 1.5)).toBe(1)
    expect(findMeasureSegmentAtTime(times, 2.9)).toBe(2)
  })

  it('clamps to the last segment when t is past the final boundary', () => {
    expect(findMeasureSegmentAtTime(times, 10)).toBe(2)
  })

  it('clamps to 0 when t is before the first boundary', () => {
    expect(findMeasureSegmentAtTime(times, -1)).toBe(0)
  })
})

describe('resolvePlayheadSegment', () => {
  const measureTimes = [0, 2, 3]

  it('computes the elapsed fraction within the current measure', () => {
    expect(resolvePlayheadSegment(1, measureTimes, 0)).toEqual({
      measureIndex: 0,
      fraction: 0.5,
    })
  })

  it('clamps fraction to [0, 1]', () => {
    expect(resolvePlayheadSegment(5, measureTimes, 0)).toEqual({
      measureIndex: 1,
      fraction: 1,
    })
    expect(resolvePlayheadSegment(-1, measureTimes, 0)).toEqual({
      measureIndex: 0,
      fraction: 0,
    })
  })

  it('applies measureIndexOffset when writtenIndices is absent', () => {
    expect(resolvePlayheadSegment(1, measureTimes, 5)).toEqual({
      measureIndex: 5,
      fraction: 0.5,
    })
  })

  it('maps through writtenIndices when provided, e.g. for D.C. al Coda repeats', () => {
    expect(resolvePlayheadSegment(1, measureTimes, 0, [7, 8, 9])).toEqual({
      measureIndex: 7,
      fraction: 0.5,
    })
  })

  it('returns fraction 0 when a segment has zero duration', () => {
    expect(resolvePlayheadSegment(2, [0, 2, 2, 3], 0)).toEqual({
      measureIndex: 2,
      fraction: 0,
    })
  })
})

describe('computePlayheadRect', () => {
  const measureRect = { x: 10, y: 20, width: 100, height: 30 }

  it('interpolates x linearly across the measure width by fraction', () => {
    expect(computePlayheadRect(measureRect, 0)).toEqual({
      x: 10,
      y: 20,
      width: 2,
      height: 30,
    })
    expect(computePlayheadRect(measureRect, 0.5)).toEqual({
      x: 60,
      y: 20,
      width: 2,
      height: 30,
    })
    expect(computePlayheadRect(measureRect, 1)).toEqual({
      x: 110,
      y: 20,
      width: 2,
      height: 30,
    })
  })
})
