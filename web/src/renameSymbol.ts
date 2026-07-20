import init, {
  list_symbols,
  rename_symbol,
  type SymbolKindOut,
  type SymbolOut,
} from 'jianpu-wasm'
import { GM_INSTRUMENTS } from './utils/gmInstruments'

let wasmReady: Promise<void> | null = null

function ensureWasmInit(): Promise<void> {
  if (!wasmReady) {
    wasmReady = init().then(() => undefined)
  }
  return wasmReady
}

/** Every renamable symbol (part/group abbreviation, section label) in `source`. */
export async function listRenameSymbols(source: string): Promise<SymbolOut[]> {
  await ensureWasmInit()
  const result = list_symbols(source, GM_INSTRUMENTS)
  return result.status === 'ok' ? result.symbols : []
}

/**
 * Whether `byteOffset` falls within `span`. The end is inclusive: a caret
 * resting right after the last character of a symbol (the natural position
 * after clicking/typing it) still counts, which matters most for
 * single-character symbols where that's the only spot after the symbol at
 * all.
 */
function spanContainsOffset(
  span: { start: number; end: number },
  byteOffset: number,
): boolean {
  return byteOffset >= span.start && byteOffset <= span.end
}

/** The symbol (if any) with an occurrence spanning `byteOffset`. */
export function symbolAtByteOffset(
  symbols: SymbolOut[],
  byteOffset: number,
): SymbolOut | null {
  for (const symbol of symbols) {
    const hit = symbol.occurrences.some((occurrence) =>
      spanContainsOffset(occurrence.hit_span, byteOffset),
    )
    if (hit) return symbol
  }
  return null
}

/** The occurrence of `symbol` (if any) spanning `byteOffset`. */
export function occurrenceAtByteOffset(symbol: SymbolOut, byteOffset: number) {
  return symbol.occurrences.find((occurrence) =>
    spanContainsOffset(occurrence.hit_span, byteOffset),
  )
}

export interface RenameTextEdit {
  start: number
  end: number
  replacement: string
}

/** Byte-offset text edits renaming every occurrence of `oldName` (of `kind`) to `newName`. */
export async function renameSymbolEdits(
  source: string,
  kind: SymbolKindOut,
  oldName: string,
  newName: string,
): Promise<RenameTextEdit[]> {
  await ensureWasmInit()
  const result = rename_symbol(source, kind, oldName, newName, GM_INSTRUMENTS)
  if (result.status !== 'ok') return []
  return result.edits.map((edit) => ({
    start: edit.span.start,
    end: edit.span.end,
    replacement: edit.replacement,
  }))
}
