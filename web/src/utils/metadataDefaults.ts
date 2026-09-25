import { jianpuWasm, type MetadataDefaults } from '../jianpuWasm'
import { ensureWasmInit } from '../wasmInit'

export type { MetadataDefaults, TextStyleDefaults } from '../jianpuWasm'

let cached: Promise<MetadataDefaults> | null = null

export function loadMetadataDefaults(): Promise<MetadataDefaults> {
  if (!cached) {
    cached = ensureWasmInit().then(() => jianpuWasm().getMetadataDefaults())
  }
  return cached
}

export async function defaultLyricsFontSize(
  rowHeight: number,
): Promise<number> {
  await ensureWasmInit()
  return jianpuWasm().getDefaultLyricsFontSize(rowHeight)
}

export async function defaultTitleFontSize(rowHeight: number): Promise<number> {
  await ensureWasmInit()
  return jianpuWasm().getDefaultTitleFontSize(rowHeight)
}

export async function defaultSubtitleFontSize(
  rowHeight: number,
): Promise<number> {
  await ensureWasmInit()
  return jianpuWasm().getDefaultSubtitleFontSize(rowHeight)
}

export async function defaultAuthorFontSize(
  rowHeight: number,
): Promise<number> {
  await ensureWasmInit()
  return jianpuWasm().getDefaultAuthorFontSize(rowHeight)
}

export async function defaultPartLegendFontSize(
  rowHeight: number,
): Promise<number> {
  await ensureWasmInit()
  return jianpuWasm().getDefaultPartLegendFontSize(rowHeight)
}

export async function defaultPageNumberFontSize(
  rowHeight: number,
): Promise<number> {
  await ensureWasmInit()
  return jianpuWasm().getDefaultPageNumberFontSize(rowHeight)
}
