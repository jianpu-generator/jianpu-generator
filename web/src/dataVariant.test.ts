/// <reference types="node" />
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { DATA_VARIANT } from './dataVariant.ts'

// `preview.css`/`index.css` are plain CSS and can't import `DATA_VARIANT`,
// so their `data-variant="..."` selectors are hand-typed string literals —
// exactly the kind of independent re-typing item 2 of
// TODO-cross-boundary-invariants.md flags. Every `data-variant="..."`
// literal actually written in either file must still be one of
// `DATA_VARIANT`'s current values: if a value here gets renamed without
// updating the CSS, the stale literal left behind fails this test instead of
// silently matching nothing at runtime.
function dataVariantLiteralsIn(cssPath: string): string[] {
  const css = readFileSync(
    fileURLToPath(new URL(cssPath, import.meta.url)),
    'utf-8',
  )
  return [...css.matchAll(/data-variant="([^"]+)"/g)].map((match) => match[1])
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
