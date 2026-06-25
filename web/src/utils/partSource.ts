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
}

function parseRhs(rhs: string): {
  mode: PartMode
  followTarget: string | null
  soundfont: SoundfontValue | null
} {
  const remaining = rhs.trim()

  let mode: PartMode
  let followTarget: string | null = null
  let soundfont: SoundfontValue | null = null

  if (remaining.startsWith('follow[')) {
    mode = 'follow'
    const match = remaining.match(/^follow\[([^\]]+)\]/)
    followTarget = match?.[1] ?? null
  } else {
    const quotePos = remaining.indexOf('"')
    let kindToken: string
    if (quotePos !== -1) {
      kindToken = remaining.slice(0, quotePos).trim()
      const afterQuote = remaining.slice(quotePos + 1)
      const closePos = afterQuote.indexOf('"')
      if (closePos !== -1) {
        soundfont = afterQuote.slice(0, closePos)
      }
    } else {
      kindToken = remaining
    }
    if (kindToken === 'notes+lyrics') {
      mode = 'notes+lyrics'
    } else if (kindToken === 'chords') {
      mode = 'chords'
    } else {
      mode = 'notes'
    }
  }

  return { mode, followTarget, soundfont }
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
      }
    }
    const { mode, followTarget, soundfont } = parseRhs(partLine.rhs)
    return {
      abbreviation: part.abbreviation,
      lineNumber: partLine.index + 1,
      mode,
      followTarget,
      soundfont,
    }
  })
}

export function updatePartDeclaration(
  source: string,
  abbreviation: string,
  newMode: PartMode,
  newFollowTarget: string | null,
  newSoundfont: SoundfontValue | null,
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
  const newLine = `${lhsWithEq} ${modeStr}${soundfontSuffix}`

  return lines.map((l, i) => (i === targetIndex ? newLine : l)).join('\n')
}
