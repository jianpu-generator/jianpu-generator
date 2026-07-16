import init, {
  get_default_lyrics_font_size,
  get_metadata_defaults,
  type MetadataDefaultsOut,
} from 'jianpu-wasm'

export type MetadataDefaults = MetadataDefaultsOut

let wasmReady: Promise<void> | null = null

function ensureWasmInit(): Promise<void> {
  if (!wasmReady) {
    wasmReady = init().then(() => undefined)
  }
  return wasmReady
}

let cached: Promise<MetadataDefaults> | null = null

export function loadMetadataDefaults(): Promise<MetadataDefaults> {
  if (!cached) {
    cached = ensureWasmInit().then(() => get_metadata_defaults())
  }
  return cached
}

export async function defaultLyricsFontSize(
  rowHeight: number,
): Promise<number> {
  await ensureWasmInit()
  return get_default_lyrics_font_size(rowHeight)
}
