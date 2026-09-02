import { describe, expect, it } from 'vitest'
import fonts from '../../fonts/fonts.json'
import { buildFontFaceCss } from './injectFontFaces.ts'

describe('buildFontFaceCss', () => {
  it('prefixes font URLs with the deploy base path, not the site root', () => {
    // Regression test: GitHub Pages serves this app under `/jianpu-generator/`
    // (see .github/workflows/pages.yml's VITE_BASE_PATH), so a `src: url(...)`
    // hardcoded to `/fonts/...` 404s in production even though it works
    // against the dev server's root base. Any hardcoded absolute path here
    // reintroduces that bug.
    const css = buildFontFaceCss('/jianpu-generator/')

    expect(css).toContain(`/jianpu-generator/fonts/${fonts.serif.filename}`)
    expect(css).not.toMatch(/url\("\/fonts\//)
  })

  it('still resolves correctly at the site root', () => {
    const css = buildFontFaceCss('/')

    expect(css).toContain(`/fonts/${fonts.serif.filename}`)
  })
})
