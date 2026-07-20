import type { SymbolOut } from 'jianpu-wasm'
import { describe, expect, it } from 'vitest'
import { symbolAtByteOffset } from './renameSymbol'

function makeSymbol(name: string, spans: Array<[number, number]>): SymbolOut {
  return {
    name,
    kind: 'abbreviation',
    occurrences: spans.map(([start, end]) => ({
      span: { start, end },
      role: 'declaration',
    })),
  }
}

describe('symbolAtByteOffset', () => {
  it('finds the symbol whose occurrence contains the offset', () => {
    const symbols = [makeSymbol('S', [[10, 11]]), makeSymbol('A', [[20, 21]])]
    expect(symbolAtByteOffset(symbols, 10)?.name).toBe('S')
    expect(symbolAtByteOffset(symbols, 20)?.name).toBe('A')
  })

  it('treats the span end as exclusive', () => {
    const symbols = [makeSymbol('S', [[10, 11]])]
    expect(symbolAtByteOffset(symbols, 11)).toBeNull()
  })

  it('returns null when no occurrence contains the offset', () => {
    const symbols = [makeSymbol('S', [[10, 11]])]
    expect(symbolAtByteOffset(symbols, 5)).toBeNull()
  })
})
