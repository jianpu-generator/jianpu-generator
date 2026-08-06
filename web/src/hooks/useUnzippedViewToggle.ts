import { useCallback } from 'react'

interface UseUnzippedViewToggleParams {
  unzippedView: boolean
  setUnzippedView: (value: boolean) => void
  formatScore: (source: string) => Promise<string>
  source: string
  handleSourceChange: (value: string) => void
}

/** The Zipped-view "Format" toolbar action, and switching from Unzipped view
 * back to Zipped view (which auto-formats the score the same way, mirroring
 * the auto-format `useUnzippedTextSnapshot` runs when Unzipped view is
 * switched on). */
export function useUnzippedViewToggle({
  unzippedView,
  setUnzippedView,
  formatScore,
  source,
  handleSourceChange,
}: UseUnzippedViewToggleParams) {
  const handleFormatScore = useCallback(() => {
    void formatScore(source).then(handleSourceChange)
  }, [formatScore, source, handleSourceChange])

  const handleToggleUnzippedView = useCallback(() => {
    if (unzippedView) {
      void formatScore(source).then((formatted) => {
        handleSourceChange(formatted)
        setUnzippedView(false)
      })
    } else {
      setUnzippedView(true)
    }
  }, [unzippedView, formatScore, source, handleSourceChange, setUnzippedView])

  return { handleFormatScore, handleToggleUnzippedView }
}
