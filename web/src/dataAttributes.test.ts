/// <reference types="node" />
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { DATA_RANGE_ACTIVE_FLAG, DATA_VARIANT } from './dataAttributes.ts'

function fileContents(path: string): string {
  return readFileSync(fileURLToPath(new URL(path, import.meta.url)), 'utf-8')
}

// `preview.css`/`index.css` are plain CSS and can't import `DATA_VARIANT`,
// so their `data-variant="..."` selectors are hand-typed string literals —
// exactly the kind of independent re-typing item 2 of
// TODO-cross-boundary-invariants.md flags. Every `data-variant="..."`
// literal actually written in either file must still be one of
// `DATA_VARIANT`'s current values: if a value here gets renamed without
// updating the CSS, the stale literal left behind fails this test instead of
// silently matching nothing at runtime.
function dataVariantLiteralsIn(cssPath: string): string[] {
  return [...fileContents(cssPath).matchAll(/data-variant="([^"]+)"/g)].map(
    (match) => match[1],
  )
}

describe('data-variant literals in CSS', () => {
  const knownValues = new Set<string>(Object.values(DATA_VARIANT))

  it.each([
    './preview.css',
    './index.css',
  ])('%s only references current DATA_VARIANT values', (cssPath) => {
    const literals = dataVariantLiteralsIn(cssPath)
    expect(literals.length).toBeGreaterThan(0)
    for (const literal of literals) {
      expect(knownValues.has(literal)).toBe(true)
    }
  })
})

// `index.css` also references the two `data-*-range-active` flags
// `previewLabelRangeHighlights.ts` toggles imperatively via
// `setAttribute`/`removeAttribute` — presence-only boolean flags, so unlike
// `data-variant` they appear bracketed with no `="..."` value
// (`[data-part-label-range-active]`). Same drift risk as `DATA_VARIANT`:
// this asserts every `data-*-range-active` literal the CSS contains is still
// one of `DATA_RANGE_ACTIVE_FLAG`'s current values.
function dataRangeActiveFlagLiteralsIn(cssPath: string): string[] {
  return [
    ...fileContents(cssPath).matchAll(/\[(data-[\w-]+-range-active)\]/g),
  ].map((match) => match[1])
}

describe('data-*-range-active literals in CSS', () => {
  const knownValues = new Set<string>(Object.values(DATA_RANGE_ACTIVE_FLAG))

  it('./index.css only references current DATA_RANGE_ACTIVE_FLAG values', () => {
    const literals = dataRangeActiveFlagLiteralsIn('./index.css')
    expect(literals.length).toBeGreaterThan(0)
    for (const literal of literals) {
      expect(knownValues.has(literal)).toBe(true)
    }
  })
})
