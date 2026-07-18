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
 * Maps a linear time `fraction` elapsed within a measure onto the pixel
 * fraction across that measure's width, via `boundaries` — the cumulative
 * pixel-weight at each of the measure's grid-column boundaries (from `0` to
 * `1`), since grid columns are duration-proportional but their rendered
 * pixel widths are density-weighted (see `measure_column_boundaries` in the
 * Rust `grid_layout` module). Falls back to `fraction` itself (linear
 * interpolation) when `boundaries` isn't available.
 */
export function mapLinearFractionToPixelFraction(
  fraction: number,
  boundaries: number[] | undefined,
): number {
  if (!boundaries || boundaries.length < 2) return fraction
  const columnCount = boundaries.length - 1
  const columnPosition = Math.min(
    columnCount,
    Math.max(0, fraction * columnCount),
  )
  const column = Math.min(columnCount - 1, Math.floor(columnPosition))
  const withinColumn = columnPosition - column
  return (
    boundaries[column] +
    withinColumn * (boundaries[column + 1] - boundaries[column])
  )
}

/**
 * Positions the playhead within a measure's click-target rect, interpolating
 * across its width by `fraction` — non-linearly via `boundaries` when given
 * (see [`mapLinearFractionToPixelFraction`]), otherwise linearly.
 */
export function computePlayheadRect(
  measureRect: PlayheadRect,
  fraction: number,
  boundaries?: number[],
): PlayheadRect {
  const pixelFraction = mapLinearFractionToPixelFraction(fraction, boundaries)
  return {
    x: measureRect.x + pixelFraction * measureRect.width,
    y: measureRect.y,
    width: 2,
    height: measureRect.height,
  }
}
