/// <reference types="node" />
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const srcDir = fileURLToPath(new URL('.', import.meta.url))

function cssFilesUnder(dir: string): string[] {
  return readdirSync(dir, { recursive: true, encoding: 'utf-8' })
    .filter((path) => path.endsWith('.css'))
    .map((path) => join(dir, path))
}

// `data-tag`/`data-variant` values are jco-generated WIT strings. Plain CSS
// can't reference those types, so selectors on them belong in
// `injectPreviewInteractionStyles.ts`, built from the typed helpers in
// `dataAttributes.ts`.
describe('plain CSS files', () => {
  it.each(
    cssFilesUnder(srcDir),
  )('%s has no data-tag/data-variant selector', (cssPath) => {
    expect(readFileSync(cssPath, 'utf-8')).not.toMatch(/data-(tag|variant)\b/)
  })
})
