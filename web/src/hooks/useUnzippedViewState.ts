import { useEffect, useState } from 'react'

/** Zipped/Unzipped view toggle state, reset to Zipped view on every file
 * switch. */
export function useUnzippedViewState(fileId: string) {
  const [unzippedView, setUnzippedView] = useState(false)

  // biome-ignore lint/correctness/useExhaustiveDependencies: fileId is the trigger for resetting the view on file switch, not read in the body
  useEffect(() => {
    setUnzippedView(false)
  }, [fileId])

  return [unzippedView, setUnzippedView] as const
}
