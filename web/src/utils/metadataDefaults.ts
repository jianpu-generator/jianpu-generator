import {
  get_default_lyrics_font_size,
  get_metadata_defaults,
  type MetadataDefaultsOut,
} from 'jianpu-wasm'
import { ensureWasmInit } from '../wasmInit'

export type MetadataDefaults = MetadataDefaultsOut

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
