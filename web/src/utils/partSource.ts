import type { PartInfo } from '../types'

export type PartMode = 'chords' | 'notes' | 'notes+lyrics' | 'follow'
// Format: "N: Instrument Name" e.g. "48: String Ensemble 1"
export type SoundfontValue = string

export interface ParsedPartDeclaration {
  abbreviation: string
  lineNumber: number
  mode: PartMode
  followTarget: string | null
  soundfont: SoundfontValue | null
  volume: number | null
  octaveOffset: number | null
}

function parseVolumeSuffix(s: string): { volume: number | null; rest: string } {
  const trimmed = s.trimEnd()
  const percentMatch = trimmed.match(/^(.*?)\s+(\d+)%$/)
  if (percentMatch) {
    const v = parseInt(percentMatch[2], 10)
    if (v >= 0 && v <= 100) {
      return { volume: v === 100 ? null : v, rest: percentMatch[1] }
    }
  }
  return { volume: null, rest: s }
}

function parseOctaveToken(token: string): number | null {
  if (/^\+\d+$/.test(token)) {
    const value = parseInt(token.slice(1), 10)
    return Number.isNaN(value) ? null : value
  }
  if (/^-\d+$/.test(token)) {
    const value = parseInt(token.slice(1), 10)
    return Number.isNaN(value) ? null : -value
  }
  return null
}

function parseOctaveSuffix(s: string): {
  octaveOffset: number | null
  rest: string
} {
  const trimmed = s.trimEnd()
  const match = trimmed.match(/^(.*?)\s+([+-]\d+)$/)
  if (match) {
    const octaveOffset = parseOctaveToken(match[2])
    if (octaveOffset !== null) {
      return {
        octaveOffset: octaveOffset === 0 ? null : octaveOffset,
        rest: match[1],
      }
    }
  }
  return { octaveOffset: null, rest: s }
}

function stripSuffixes(s: string): {
  volume: number | null
  octaveOffset: number | null
  rest: string
} {
  let rest = s
  let volume: number | null = null
  let octaveOffset: number | null = null

  for (let pass = 0; pass < 2; pass++) {
    const volumeResult = parseVolumeSuffix(rest)
    rest = volumeResult.rest
    if (volumeResult.volume !== null) volume = volumeResult.volume

    const octaveResult = parseOctaveSuffix(rest)
    rest = octaveResult.rest
    if (octaveResult.octaveOffset !== null) {
      octaveOffset = octaveResult.octaveOffset
    }
  }

  return { volume, octaveOffset, rest }
}

function parseRhs(rhs: string): {
  mode: PartMode
  followTarget: string | null
  soundfont: SoundfontValue | null
  volume: number | null
  octaveOffset: number | null
} {
  const remaining = rhs.trim()

  let mode: PartMode
  let followTarget: string | null = null
  let soundfont: SoundfontValue | null = null
  let volume: number | null = null
  let octaveOffset: number | null = null

  if (remaining.startsWith('follow[')) {
    const suffixes = stripSuffixes(remaining)
    volume = suffixes.volume
    octaveOffset = suffixes.octaveOffset
    mode = 'follow'
    const match = suffixes.rest.match(/^follow\[([^\]]+)\]/)
    followTarget = match?.[1] ?? null
  } else {
    const suffixes = stripSuffixes(remaining)
    volume = suffixes.volume
    octaveOffset = suffixes.octaveOffset
    const trimmedRhs = suffixes.rest.trim()
    const quotePos = trimmedRhs.indexOf('"')
    let kindToken: string
    if (quotePos !== -1) {
      kindToken = trimmedRhs.slice(0, quotePos).trim()
      const afterQuote = trimmedRhs.slice(quotePos + 1)
      const closePos = afterQuote.indexOf('"')
      if (closePos !== -1) {
        soundfont = afterQuote.slice(0, closePos)
      }
    } else {
      kindToken = trimmedRhs
    }
    if (kindToken === 'notes+lyrics') {
      mode = 'notes+lyrics'
    } else if (kindToken === 'chords') {
      mode = 'chords'
    } else {
      mode = 'notes'
    }
  }

  return { mode, followTarget, soundfont, volume, octaveOffset }
}

function findPartsSection(lines: string[]): {
  startIndex: number
  partLines: { index: number; rhs: string }[]
} {
  const startIndex = lines.findIndex((line) => line.trim() === '# parts')
  if (startIndex === -1) return { startIndex: -1, partLines: [] }

  const partLines: { index: number; rhs: string }[] = []
  for (let i = startIndex + 1; i < lines.length; i++) {
    const trimmed = lines[i].trim()
    if (trimmed === '') continue
    if (trimmed.startsWith('#')) break
    const eqIndex = lines[i].indexOf('=')
    if (eqIndex === -1) continue
    partLines.push({ index: i, rhs: lines[i].slice(eqIndex + 1) })
  }

  return { startIndex, partLines }
}

export function parsePartDeclarations(
  source: string,
  parts: PartInfo[],
): ParsedPartDeclaration[] {
  const lines = source.split('\n')
  const { partLines } = findPartsSection(lines)

  return parts.map((part, index) => {
    const partLine = partLines[index]
    if (!partLine) {
      return {
        abbreviation: part.abbreviation,
        lineNumber: 0,
        mode: 'notes' satisfies PartMode,
        followTarget: null,
        soundfont: null,
        volume: null,
        octaveOffset: null,
      }
    }
    const { mode, followTarget, soundfont, volume, octaveOffset } = parseRhs(
      partLine.rhs,
    )
    return {
      abbreviation: part.abbreviation,
      lineNumber: partLine.index + 1,
      mode,
      followTarget,
      soundfont,
      volume,
      octaveOffset,
    }
  })
}

export function updatePartDeclaration(
  source: string,
  abbreviation: string,
  newMode: PartMode,
  newFollowTarget: string | null,
  newSoundfont: SoundfontValue | null,
  newVolume: number | null,
  newOctaveOffset: number | null,
): string {
  const lines = source.split('\n')
  const partsIndex = lines.findIndex((line) => line.trim() === '# parts')
  if (partsIndex === -1) return source

  let targetIndex = -1
  for (let i = partsIndex + 1; i < lines.length; i++) {
    const trimmed = lines[i].trim()
    if (trimmed === '') continue
    if (trimmed.startsWith('# ')) break
    const eqPos = lines[i].indexOf('=')
    if (eqPos === -1) continue
    const lhs = lines[i].slice(0, eqPos).trim()
    const bracketStart = lhs.lastIndexOf('[')
    const lineAbbr =
      bracketStart !== -1 ? lhs.slice(bracketStart + 1).replace(/\]$/, '') : lhs
    if (lineAbbr === abbreviation) {
      targetIndex = i
      break
    }
  }
  if (targetIndex === -1) return source

  const line = lines[targetIndex]
  const eqPos = line.indexOf('=')
  if (eqPos === -1) return source
  const lhsWithEq = line.slice(0, eqPos + 1)

  const modeStr =
    newMode === 'follow' ? `follow[${newFollowTarget ?? ''}]` : newMode
  const soundfontSuffix = newSoundfont != null ? ` "${newSoundfont}"` : ''
  const volumeSuffix =
    newVolume != null && newVolume !== 100 ? ` ${newVolume}%` : ''
  const octaveSuffix =
    newOctaveOffset != null && newOctaveOffset !== 0
      ? ` ${newOctaveOffset > 0 ? `+${newOctaveOffset}` : newOctaveOffset}`
      : ''
  const newLine = `${lhsWithEq} ${modeStr}${soundfontSuffix}${volumeSuffix}${octaveSuffix}`

  return lines.map((l, i) => (i === targetIndex ? newLine : l)).join('\n')
}
