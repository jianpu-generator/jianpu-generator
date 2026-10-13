import type { ReactElement } from 'react'
import { splitByQuery } from './highlightMatches'

/** Text with the occurrences of `query` wrapped in <mark>. */
export function Highlighted(props: {
  text: string
  query: string
}): ReactElement {
  const segments = splitByQuery(props.text, props.query)
  return (
    <>
      {segments.map((segment, index) =>
        segment.matched ? (
          // biome-ignore lint/suspicious/noArrayIndexKey: segments are positional and never reorder
          <mark key={index}>{segment.text}</mark>
        ) : (
          // biome-ignore lint/suspicious/noArrayIndexKey: segments are positional and never reorder
          <span key={index}>{segment.text}</span>
        ),
      )}
    </>
  )
}
