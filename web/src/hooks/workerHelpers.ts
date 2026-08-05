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

/** A declared part's primary block of Unzipped view text and the
 * per-measure byte ranges within it, as returned by
 * `extract_unzipped_text`. */
interface PartMeasureRangesLike {
  abbreviation: string
  ranges: { start: number; end: number }[]
}

/** One declared part's tagged `[Abbrev:lyrics:N]` verse block and its
 * per-measure byte ranges, as returned by `extract_unzipped_text`. */
interface LyricsVerseRangesLike {
  abbreviation: string
  verseNumber: number
  ranges: { start: number; end: number }[]
}

/** One block of Unzipped view text — either a part's primary block
 * (`verseNumber: null`) or a tagged lyrics verse block. */
interface UnzippedTextBlock {
  abbreviation: string
  verseNumber: number | null
  ranges: { start: number; end: number }[]
}

/** Combines primary and lyrics-verse blocks into one offset-sorted list.
 * Every declared part contributes at most one primary block plus, for
 * `notes+lyrics`/`lyrics`-kind parts, zero or more tagged verse blocks —
 * these no longer form one contiguous block per part (a part can now have
 * several, interleaved with other parts' blocks in emission order), so
 * lookup can't assume a part's bounds run up to the next part's start. */
function collectUnzippedTextBlocks(
  partMeasureRanges: PartMeasureRangesLike[],
  lyricsVerseRanges: LyricsVerseRangesLike[],
): UnzippedTextBlock[] {
  const primary: UnzippedTextBlock[] = partMeasureRanges.map((part) => ({
    abbreviation: part.abbreviation,
    verseNumber: null,
    ranges: part.ranges,
  }))
  const verses: UnzippedTextBlock[] = lyricsVerseRanges.map((verse) => ({
    abbreviation: verse.abbreviation,
    verseNumber: verse.verseNumber,
    ranges: verse.ranges,
  }))
  return [...primary, ...verses]
    .filter((block) => block.ranges[0] !== undefined)
    .sort((a, b) => a.ranges[0].start - b.ranges[0].start)
}

/** Unzipped view text is a sequence of blocks (see
 * `collectUnzippedTextBlocks`), each starting at its first range's start
 * and extending up to (but not including) the next block's start, since
 * blocks are contiguous and offset-sorted; the last block extends to
 * infinity. A byte offset into the generated text first identifies which
 * block it falls in, then which of that block's measures it falls in (or is
 * nearest to, for offsets landing in inter-token whitespace). */
export function measureRangeInUnzippedText(
  partMeasureRanges: PartMeasureRangesLike[],
  cursorOffset: number,
  lyricsVerseRanges: LyricsVerseRangesLike[] = [],
): { start: number; end: number } | null {
  const blocks = collectUnzippedTextBlocks(partMeasureRanges, lyricsVerseRanges)
  const block = findBlockForOffset(blocks, cursorOffset)
  if (block === null) return null
  const measureIndex = findMeasureIndexForOffset(block.ranges, cursorOffset)
  if (measureIndex === null) return null
  return { start: measureIndex, end: measureIndex }
}

function findBlockForOffset(
  blocks: UnzippedTextBlock[],
  cursorOffset: number,
): UnzippedTextBlock | null {
  if (blocks.length === 0) return null
  let result = blocks[0]
  for (const block of blocks) {
    if (cursorOffset < block.ranges[0].start) break
    result = block
  }
  return result
}

function findMeasureIndexForOffset(
  ranges: { start: number; end: number }[],
  cursorOffset: number,
): number | null {
  if (ranges.length === 0) return null
  // Binary-search for the measure whose [start, end) contains cursorOffset;
  // fall back to the nearest measure for offsets in inter-token whitespace.
  let low = 0
  let high = ranges.length - 1
  while (low <= high) {
    const mid = (low + high) >> 1
    const range = ranges[mid]
    if (cursorOffset < range.start) {
      high = mid - 1
    } else if (cursorOffset >= range.end) {
      low = mid + 1
    } else {
      return mid
    }
  }
  // `low` is now the first measure whose range starts after cursorOffset
  // (or ranges.length if past the end); the nearest measure is either that
  // one or the one before it.
  const after = Math.min(low, ranges.length - 1)
  const before = Math.max(low - 1, 0)
  const distanceTo = (index: number) => {
    const range = ranges[index]
    if (cursorOffset < range.start) return range.start - cursorOffset
    if (cursorOffset >= range.end) return cursorOffset - (range.end - 1)
    return 0
  }
  return distanceTo(before) <= distanceTo(after) ? before : after
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
