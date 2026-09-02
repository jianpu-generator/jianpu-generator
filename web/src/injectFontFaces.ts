import fonts from '../../fonts/fonts.json'

// index.css can't import a JSON/JS constant, so the `@font-face` rules it
// used to hold statically are built here instead, straight from
// `fonts/fonts.json` — the single source of truth for which font file backs
// each `FontFamily` role (see that file's own comments, and `src/fonts.rs`
// on the Rust side). Call this once, before the app renders, so there's no
// extra FOUC window versus the static rules it replaces.

const FORMAT_BY_EXTENSION: Record<string, string> = {
  '.ttf': 'truetype',
  '.otf': 'opentype',
}

function fontFormat(filename: string): string {
  const extension = filename.slice(filename.lastIndexOf('.'))
  const format = FORMAT_BY_EXTENSION[extension]
  if (format == null) {
    throw new Error(
      `injectFontFaces: unknown font file extension "${extension}"`,
    )
  }
  return format
}

/** Family name as it appears bare (without quotes/fallback) in `familyCss`,
 * e.g. `"TW-Kai", sans-serif"` -> `TW-Kai` — that's the name `@font-face`'s
 * own `font-family` declaration needs. */
function bareFamilyName(familyCss: string): string {
  const match = /^"([^"]+)"/.exec(familyCss)
  if (match == null) {
    throw new Error(`injectFontFaces: unexpected familyCss "${familyCss}"`)
  }
  return match[1]
}

/** Builds the `@font-face` rules as a plain string — split out from
 * `injectFontFaces` so the URL-prefixing logic is unit-testable without a
 * DOM (see injectFontFaces.test.ts). `baseUrl` must be Vite's
 * `import.meta.env.BASE_URL`-shaped value: an absolute path ending in `/`. */
export function buildFontFaceCss(baseUrl: string): string {
  // Only `serif`/`sansSerif` get a `@font-face` rule, matching the static
  // rules this replaces: the preview SVG renders monospace-role glyphs with
  // the plain CSS `monospace` keyword rather than a `FontFamilyOut` family
  // stack (see `textFontFamily` in PreviewSvgRenderer.tsx), so there's
  // nothing in the browser for a `Monospace` `@font-face` rule to back.
  const roles = [fonts.serif, fonts.sansSerif]
  // Two roles can point at the same font file/family (e.g. while
  // experimenting with a single typeface for both) — dedupe by family name
  // so that doesn't produce two identical `@font-face` rules.
  const seenFamilyNames = new Set<string>()
  return roles
    .map((role) => {
      const familyName = bareFamilyName(role.familyCss)
      const format = fontFormat(role.filename)
      return { familyName, format, filename: role.filename }
    })
    .filter(({ familyName }) => {
      if (seenFamilyNames.has(familyName)) {
        return false
      }
      seenFamilyNames.add(familyName)
      return true
    })
    .map(
      ({ familyName, format, filename }) => `@font-face {
  font-family: "${familyName}";
  src: url("${baseUrl}fonts/${filename}") format("${format}");
  font-display: swap;
}`,
    )
    .join('\n\n')
}

export function injectFontFaces(
  baseUrl: string = import.meta.env.BASE_URL,
): void {
  const style = document.createElement('style')
  style.textContent = buildFontFaceCss(baseUrl)
  document.head.appendChild(style)
}
