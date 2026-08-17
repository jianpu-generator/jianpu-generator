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

export function downloadPdf(bytes: ArrayBuffer, filename: string) {
  const url = URL.createObjectURL(
    new Blob([bytes], { type: 'application/pdf' }),
  )
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  anchor.click()
  URL.revokeObjectURL(url)
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

export function downloadMidi(bytes: ArrayBuffer, filename: string) {
  const url = URL.createObjectURL(new Blob([bytes], { type: 'audio/midi' }))
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  anchor.click()
  URL.revokeObjectURL(url)
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

export function zipFilenameFromActiveFile(
  activeFile: string,
  suffix?: string,
): string {
  const base = activeFile.endsWith('.jianpu')
    ? activeFile.replace(/\.jianpu$/, '')
    : activeFile
  return suffix ? `${base} (${suffix}).zip` : `${base}.zip`
}

export function baseNameFromActiveFile(activeFile: string): string {
  if (activeFile.endsWith('.jianpu')) {
    return activeFile.replace(/\.jianpu$/, '')
  }
  return activeFile
}

export function downloadZip(bytes: ArrayBuffer, filename: string) {
  const url = URL.createObjectURL(
    new Blob([bytes], { type: 'application/zip' }),
  )
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  anchor.click()
  URL.revokeObjectURL(url)
}
