import { useEffect } from 'react'

function findMeasureSegmentAtTime(times: number[], t: number): number {
  for (let i = times.length - 2; i >= 0; i--) {
    if (t >= times[i]) return i
  }
  return 0
}

/**
 * Imperatively drives an SVG playhead `<rect>` across measures in sync with
 * `audio`'s playback position, using `measureTimes` (seconds per measure
 * boundary, offset by `measureIndexOffset`) to locate each measure's x/width
 * via its existing click-target rect. Runs outside React state/rendering
 * (rAF, direct attribute writes) since it updates every animation frame.
 */
export function usePlayhead(
  containerRef: React.RefObject<HTMLDivElement | null>,
  audio: HTMLAudioElement | null | undefined,
  measureTimes: number[] | undefined,
  measureIndexOffset: number,
) {
  useEffect(() => {
    const container = containerRef.current
    if (!audio || !container || !measureTimes || measureTimes.length < 2) {
      return
    }

    const playhead = document.createElementNS(
      'http://www.w3.org/2000/svg',
      'rect',
    )
    playhead.setAttribute('data-testid', 'measure-playhead')
    playhead.setAttribute('width', '2')
    playhead.setAttribute('fill', 'rgba(220,38,38,0.85)')
    playhead.style.pointerEvents = 'none'
    let currentSvg: SVGSVGElement | null = null
    let rafId: number | null = null

    const updatePosition = () => {
      const t = audio.currentTime
      const segment = findMeasureSegmentAtTime(measureTimes, t)
      const measureIndex = measureIndexOffset + segment
      const group = container.querySelector<SVGGElement>(
        `[data-tag="measure"][data-measure-index="${measureIndex}"]`,
      )
      const targetRect = group?.querySelector<SVGRectElement>(
        'rect[data-variant="measure-click-target-rect"]',
      )
      if (!group || !targetRect) return
      const svg = group.closest('svg')
      if (svg && svg !== currentSvg) {
        playhead.remove()
        svg.appendChild(playhead)
        currentSvg = svg
      }
      const x = Number.parseFloat(targetRect.getAttribute('x') ?? '0')
      const y = Number.parseFloat(targetRect.getAttribute('y') ?? '0')
      const width = Number.parseFloat(targetRect.getAttribute('width') ?? '0')
      const height = Number.parseFloat(targetRect.getAttribute('height') ?? '0')
      const segStart = measureTimes[segment]
      const segEnd = measureTimes[segment + 1] ?? segStart
      const fraction =
        segEnd > segStart
          ? Math.min(1, Math.max(0, (t - segStart) / (segEnd - segStart)))
          : 0
      playhead.setAttribute('x', String(x + fraction * width))
      playhead.setAttribute('y', String(y))
      playhead.setAttribute('height', String(height))
    }

    const tick = () => {
      updatePosition()
      rafId = requestAnimationFrame(tick)
    }
    const start = () => {
      if (rafId === null) rafId = requestAnimationFrame(tick)
    }
    const stop = () => {
      if (rafId !== null) {
        cancelAnimationFrame(rafId)
        rafId = null
      }
      playhead.remove()
      currentSvg = null
    }

    audio.addEventListener('play', start)
    audio.addEventListener('pause', stop)
    audio.addEventListener('ended', stop)
    if (!audio.paused) start()

    return () => {
      audio.removeEventListener('play', start)
      audio.removeEventListener('pause', stop)
      audio.removeEventListener('ended', stop)
      stop()
    }
  }, [containerRef, audio, measureTimes, measureIndexOffset])
}
