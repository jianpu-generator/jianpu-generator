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

/** The symbol (if any) with an occurrence spanning `byteOffset`. */
export function symbolAtByteOffset(
  symbols: SymbolOut[],
  byteOffset: number,
): SymbolOut | null {
  for (const symbol of symbols) {
    const hit = symbol.occurrences.some(
      (occurrence) =>
        byteOffset >= occurrence.span.start && byteOffset < occurrence.span.end,
    )
    if (hit) return symbol
  }
  return null
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
