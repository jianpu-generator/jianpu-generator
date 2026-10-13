export interface TextSegment {
  text: string
  matched: boolean
}

/** Splits `text` into runs, marking the case-insensitive occurrences of
 * `query`. An empty query yields one unmatched run. */
export function splitByQuery(text: string, query: string): TextSegment[] {
  if (query === '') return [{ text, matched: false }]
  const lowerText = text.toLowerCase()
  const lowerQuery = query.toLowerCase()
  const segments: TextSegment[] = []
  let cursor = 0
  for (
    let found = lowerText.indexOf(lowerQuery);
    found >= 0;
    found = lowerText.indexOf(lowerQuery, cursor)
  ) {
    if (found > cursor) {
      segments.push({ text: text.slice(cursor, found), matched: false })
    }
    segments.push({
      text: text.slice(found, found + query.length),
      matched: true,
    })
    cursor = found + query.length
  }
  if (cursor < text.length) {
    segments.push({ text: text.slice(cursor), matched: false })
  }
  return segments
}
