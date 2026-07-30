import { useEffect, useState } from 'react'

/** Below this width, the editor/preview panes stack instead of sitting side-by-side. */
export const MOBILE_BREAKPOINT_QUERY = '(max-width: 768px)'

/** Tracks whether `query` currently matches, updating on viewport changes. */
export function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(() => window.matchMedia(query).matches)

  useEffect(() => {
    const mediaQueryList = window.matchMedia(query)
    const listener = () => setMatches(mediaQueryList.matches)
    listener()
    mediaQueryList.addEventListener('change', listener)
    return () => mediaQueryList.removeEventListener('change', listener)
  }, [query])

  return matches
}
