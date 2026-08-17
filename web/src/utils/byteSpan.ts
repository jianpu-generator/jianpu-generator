const utf8LenCache = new Map<string, number>()
const utf8Encoder = new TextEncoder()

function utf8ByteLengthOfCodePoint(ch: string): number {
  const cached = utf8LenCache.get(ch)
  if (cached != null) return cached
  const len = utf8Encoder.encode(ch).length
  utf8LenCache.set(ch, len)
  return len
}

/**
 * Map a JS string index (UTF-16 code unit count from Monaco) to a UTF-8 byte offset
 * suitable for passing to the Rust parser.
 *
 * Monaco's `ITextModel` normalizes line endings independently of the source
 * text it was created from — in particular, a model created with empty
 * content (as happens transiently on file switch) can default to CRLF
 * (`\r\n`) even though the Rust/wasm core always generates and measures its
 * byte offsets against
 * LF-only (`\n`) text. `charIndex`, from `model.getOffsetAt`, is a UTF-16
 * code unit count into whatever the model's *actual* buffer contains, so if
 * that buffer is CRLF, `source` (from `model.getValue()`) is too, and it
 * carries one extra `\r` byte per line preceding `charIndex` that the Rust
 * side's offsets don't account for. Stripping `\r` from the slice before
 * encoding — rather than requiring every model to be forced LF-only, which
 * risks emitting a spurious `onDidChangeModelContent` event that gets
 * misread as a real edit — keeps this conversion correct regardless of the
 * model's line-ending preference.
 */
export function stringIndexToByteOffset(
  source: string,
  charIndex: number,
): number {
  const lfOnly = source.slice(0, charIndex).replace(/\r/g, '')
  return utf8Encoder.encode(lfOnly).length
}

/**
 * Map a UTF-8 byte offset (from the Rust parser) to a JS string index (UTF-16).
 */
export function byteOffsetToStringIndex(
  source: string,
  byteOffset: number,
): number {
  let bytePos = 0
  let strIndex = 0

  for (const ch of source) {
    const cpByteLen = utf8ByteLengthOfCodePoint(ch)
    const cpEndByte = bytePos + cpByteLen
    if (cpEndByte <= byteOffset) {
      strIndex += ch.length
      bytePos = cpEndByte
      continue
    }
    break
  }

  return strIndex
}
