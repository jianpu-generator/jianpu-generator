import { useCallback, useState } from 'react'
import {
  MOBILE_BREAKPOINT_QUERY,
  useMediaQuery,
} from '../../hooks/useMediaQuery'

export type PartStreamDiffLayout = 'side-by-side' | 'inline'

const STORAGE_KEY = 'jianpu:part-stream-diff-layout:v1'

function load(): PartStreamDiffLayout | null {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    return stored === 'side-by-side' || stored === 'inline' ? stored : null
  } catch {
    return null
  }
}

function save(layout: PartStreamDiffLayout) {
  try {
    localStorage.setItem(STORAGE_KEY, layout)
  } catch {
    // Storage may be blocked; the choice just won't persist.
  }
}

/** Whether the part stream diff is shown side by side or inline. Until the
 * user picks one (remembered in this browser only), it follows the viewport:
 * inline on mobile, side by side otherwise. */
export function usePartStreamDiffLayout() {
  const isMobile = useMediaQuery(MOBILE_BREAKPOINT_QUERY)
  const [chosen, setChosen] = useState<PartStreamDiffLayout | null>(load)
  const layout = chosen ?? (isMobile ? 'inline' : 'side-by-side')
  const toggle = useCallback(() => {
    const next = layout === 'inline' ? 'side-by-side' : 'inline'
    save(next)
    setChosen(next)
  }, [layout])
  return { layout, toggle }
}
