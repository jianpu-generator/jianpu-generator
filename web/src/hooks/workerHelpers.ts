import { findIndex, findLastIndex } from 'remeda'
import type { MeasureSpan, PartInfo } from '../types'

export function measureRangeInSpan(
  spans: MeasureSpan[],
  startLine: number,
  endLine: number,
): { start: number; end: number } | null {
  const overlaps = (span: MeasureSpan) =>
    span.start_line <= endLine && span.end_line >= startLine
  const start = findIndex(spans, overlaps)
  const end = findLastIndex(spans, overlaps)
  return start === -1 ? null : { start, end }
}

/**
 * Like `measureRangeInSpan`, but also resolves `revealLine` to its own
 * measure index and attaches it as `revealMeasureIndex` — the measure the
 * preview should scroll to for this selection, when it differs from
 * `start` (e.g. a `# sequence` chain selection whose envelope start, in
 * document order, isn't the entry the user actually navigated to). Falls
 * back to `start` when `revealLine` doesn't resolve to a measure of its
 * own.
 *
 * `measureRanges`, when given, is folded in as `highlightRanges` — the
 * exact disjoint measure ranges to highlight in the SVG preview for a `#
 * sequence` chain selection, bypassing `measureRangeInSpan`'s single-span
 * derivation entirely (it can't represent "C and A but not B in between").
 */
export function measureRangeInSpanWithReveal(
  spans: MeasureSpan[],
  startLine: number,
  endLine: number,
  revealLine: number,
  measureRanges?: { start: number; end: number }[],
): {
  start: number
  end: number
  revealMeasureIndex: number
  highlightRanges?: { start: number; end: number }[]
} | null {
  const range = measureRangeInSpan(spans, startLine, endLine)
  if (!range) return null
  const revealMeasureIndex =
    measureRangeInSpan(spans, revealLine, revealLine)?.start ?? range.start
  return {
    ...range,
    revealMeasureIndex,
    ...(measureRanges ? { highlightRanges: measureRanges } : {}),
  }
}

export function enabledTracksForRender(
  parts: PartInfo[],
  disabledParts: ReadonlySet<string>,
): string[] | undefined {
  if (parts.length === 0) return undefined
  const enabled = parts
    .filter((part) => !disabledParts.has(part.abbreviation))
    .map((part) => part.abbreviation)
  if (enabled.length === parts.length) return undefined
  return enabled
}

export function enabledPartNamesForFilename(
  parts: PartInfo[],
  disabledParts: ReadonlySet<string>,
): string[] | undefined {
  if (parts.length === 0) return undefined
  const enabled = parts
    .filter((part) => !disabledParts.has(part.abbreviation))
    .map((part) => part.display_name)
  if (enabled.length === parts.length) return undefined
  return enabled
}

export function disabledLyricsForRender(
  parts: PartInfo[],
  disabledLyrics: ReadonlySet<string>,
): string[] | undefined {
  const lyricParts = parts.filter((part) => part.has_lyrics)
  if (lyricParts.length === 0) return undefined
  const disabled = lyricParts
    .filter((part) => disabledLyrics.has(part.abbreviation))
    .map((part) => part.abbreviation)
  if (disabled.length === 0) return undefined
  return disabled
}

/** Wraps export bytes in an object URL of the given MIME type — the URL is
 * the caller's to revoke once it's no longer needed (see `requestDownload`/
 * `cancelPendingDownload`/`confirmPendingDownload` in
 * `useJianpuWorkerState.ts`). */
export function objectUrlForBytes(bytes: ArrayBuffer, mimeType: string) {
  return URL.createObjectURL(new Blob([bytes], { type: mimeType }))
}

/** Fires a browser download for `url` under `filename` via a throwaway
 * anchor click — the one primitive every export format's download funnels
 * through. Does not revoke `url`; the caller owns its lifetime. */
export function triggerAnchorDownload(url: string, filename: string) {
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  anchor.click()
}

// When only some parts are enabled, mark that in the filename so exports
// of different toggle combinations don't silently overwrite each other.
export function withEnabledPartsSuffix(
  filename: string,
  enabledPartNames: string[] | undefined,
): string {
  if (!enabledPartNames || enabledPartNames.length === 0) return filename
  const suffix = enabledPartNames.join(', ')
  const dotIndex = filename.lastIndexOf('.')
  if (dotIndex === -1) return `${filename} (${suffix})`
  return `${filename.slice(0, dotIndex)} (${suffix})${filename.slice(dotIndex)}`
}

export function pdfFilenameFromActiveFile(
  activeFile: string,
  enabledPartNames?: string[],
): string {
  const base = activeFile.endsWith('.jianpu')
    ? activeFile.replace(/\.jianpu$/, '.pdf')
    : `${activeFile}.pdf`
  return withEnabledPartsSuffix(base, enabledPartNames)
}

export function midiFilenameFromActiveFile(
  activeFile: string,
  enabledPartNames?: string[],
): string {
  const base = activeFile.endsWith('.jianpu')
    ? activeFile.replace(/\.jianpu$/, '.mid')
    : `${activeFile}.mid`
  return withEnabledPartsSuffix(base, enabledPartNames)
}

export function wavFilenameFromActiveFile(
  activeFile: string,
  enabledPartNames?: string[],
): string {
  const base = activeFile.endsWith('.jianpu')
    ? activeFile.replace(/\.jianpu$/, '.wav')
    : `${activeFile}.wav`
  return withEnabledPartsSuffix(base, enabledPartNames)
}

export function mp3FilenameFromActiveFile(
  activeFile: string,
  enabledPartNames?: string[],
): string {
  const base = activeFile.endsWith('.jianpu')
    ? activeFile.replace(/\.jianpu$/, '.mp3')
    : `${activeFile}.mp3`
  return withEnabledPartsSuffix(base, enabledPartNames)
}

export function zipFilenameFromActiveFile(
  activeFile: string,
  suffix?: string,
): string {
  const base = activeFile.endsWith('.jianpu')
    ? activeFile.replace(/\.jianpu$/, '')
    : activeFile
  return suffix ? `${base} (${suffix}).zip` : `${base}.zip`
}

/** Posts `payload` to `worker` only after the browser has had a chance to
 * paint — use this right after flipping an "…Exporting" flag to `true` so
 * the resulting loading toast is guaranteed at least one visible frame
 * before the worker's response can flip it back to `false`. Without this,
 * a fast enough response (e.g. encoding a tiny fixture, or a paint delayed
 * by CPU contention from other work) can let both state flips land in the
 * same frame, so the toast never actually renders. `requestAnimationFrame`
 * fires right before the next paint, so by the time it runs, any commit
 * from the earlier `setState` call has already had its chance to paint. */
export function postAfterPaint(worker: Worker, payload: unknown) {
  requestAnimationFrame(() => worker.postMessage(payload))
}

export function baseNameFromActiveFile(activeFile: string): string {
  if (activeFile.endsWith('.jianpu')) {
    return activeFile.replace(/\.jianpu$/, '')
  }
  return activeFile
}
