export interface PlayheadRect {
  x: number
  y: number
  width: number
  height: number
}

export function findMeasureSegmentAtTime(times: number[], t: number): number {
  for (let i = times.length - 2; i >= 0; i--) {
    if (t >= times[i]) return i
  }
  return 0
}

/**
 * Maps a playback time to the current measure's written index and the
 * fraction of that measure's duration elapsed (clamped to [0, 1]).
 */
export function resolvePlayheadSegment(
  t: number,
  measureTimes: number[],
  measureIndexOffset: number,
  writtenIndices?: number[],
): { measureIndex: number; fraction: number } {
  const segment = findMeasureSegmentAtTime(measureTimes, t)
  const measureIndex = writtenIndices?.[segment] ?? measureIndexOffset + segment
  const segStart = measureTimes[segment]
  const segEnd = measureTimes[segment + 1] ?? segStart
  const fraction =
    segEnd > segStart
      ? Math.min(1, Math.max(0, (t - segStart) / (segEnd - segStart)))
      : 0
  return { measureIndex, fraction }
}

/**
 * Positions the playhead within a measure's click-target rect, linearly
 * interpolating across its width by `fraction`.
 */
export function computePlayheadRect(
  measureRect: PlayheadRect,
  fraction: number,
): PlayheadRect {
  return {
    x: measureRect.x + fraction * measureRect.width,
    y: measureRect.y,
    width: 2,
    height: measureRect.height,
  }
}
