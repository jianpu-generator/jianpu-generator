import { findIndex, findLastIndex } from 'remeda'
import type { MeasureSpan, PartInfo } from '../types'

export function measureRangeInSpan(
  spans: MeasureSpan[],
  selStart: number,
  selEnd: number,
): { start: number; end: number } | null {
  const effective = selStart === selEnd ? selEnd + 1 : selEnd
  const overlaps = (span: MeasureSpan) =>
    span.start < effective && span.end > selStart
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

export function pdfFilenameFromActiveFile(activeFile: string): string {
  if (activeFile.endsWith('.jianpu')) {
    return activeFile.replace(/\.jianpu$/, '.pdf')
  }
  return `${activeFile}.pdf`
}

export function zipFilenameFromActiveFile(activeFile: string): string {
  if (activeFile.endsWith('.jianpu')) {
    return activeFile.replace(/\.jianpu$/, '.zip')
  }
  return `${activeFile}.zip`
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
