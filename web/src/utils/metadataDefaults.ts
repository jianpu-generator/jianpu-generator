import {
  get_default_author_font_size,
  get_default_lyrics_font_size,
  get_default_part_legend_font_size,
  get_default_subtitle_font_size,
  get_default_title_font_size,
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

export async function defaultTitleFontSize(rowHeight: number): Promise<number> {
  await ensureWasmInit()
  return get_default_title_font_size(rowHeight)
}

export async function defaultSubtitleFontSize(
  rowHeight: number,
): Promise<number> {
  await ensureWasmInit()
  return get_default_subtitle_font_size(rowHeight)
}

export async function defaultAuthorFontSize(
  rowHeight: number,
): Promise<number> {
  await ensureWasmInit()
  return get_default_author_font_size(rowHeight)
}

export async function defaultPartLegendFontSize(
  rowHeight: number,
): Promise<number> {
  await ensureWasmInit()
  return get_default_part_legend_font_size(rowHeight)
}
