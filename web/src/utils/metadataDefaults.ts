import {
  type FontFamilyDefault,
  jianpuWasm,
  type MetadataDefaults,
  type TextStyleDefaults,
} from '../jianpuWasm'
import { ensureWasmInit } from '../wasmInit'
import type {
  FontFamilyValue,
  TextStyleComponent,
  TextStyleFields,
} from './textStyleFields'

export type { MetadataDefaults, TextStyleDefaults } from '../jianpuWasm'

/** A kind's fully-resolved default style, in the `.jianpu` metadata
 * syntax's own component vocabulary (`TextStyleFields`) — what the metadata
 * editor shows as each unset component's placeholder. */
export type TextStyleDefaultFields = {
  [Component in TextStyleComponent]: NonNullable<TextStyleFields[Component]>
}

const FONT_FAMILY_VALUE_FROM_DEFAULT: Record<
  FontFamilyDefault,
  FontFamilyValue
> = {
  serif: 'serif',
  'sans-serif': 'sans_serif',
  monospace: 'monospace',
}

export function textStyleDefaultFields(
  defaults: TextStyleDefaults,
): TextStyleDefaultFields {
  return {
    font_size: defaults.fontSize,
    horizontal_padding_pt: defaults.horizontalPaddingPt,
    vertical_padding_pt: defaults.verticalPaddingPt,
    bold: defaults.bold,
    italic: defaults.italic,
    underline: defaults.underline,
    font_family: FONT_FAMILY_VALUE_FROM_DEFAULT[defaults.fontFamily],
  }
}

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
