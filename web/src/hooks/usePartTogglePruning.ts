import { useEffect } from 'react'
import type { PartInfo } from '../types'

type SetStringSet = (updater: (prev: Set<string>) => Set<string>) => void

/** Removes stale entries from the disabled-parts/soloed-parts sets whenever the
 * set of known parts changes (e.g. after an edit that adds or removes a part). */
export function usePartTogglePruning(
  parts: PartInfo[],
  setDisabledParts: SetStringSet,
  setSoloedParts: SetStringSet,
) {
  useEffect(() => {
    if (parts.length === 0) return

    const abbreviations = new Set(parts.map((part) => part.abbreviation))
    setDisabledParts((prev) => {
      const next = new Set(
        [...prev].filter((abbreviation) => abbreviations.has(abbreviation)),
      )
      return next.size === prev.size ? prev : next
    })
  }, [parts, setDisabledParts])

  useEffect(() => {
    if (parts.length === 0) return
    const abbreviations = new Set(parts.map((part) => part.abbreviation))
    setSoloedParts((prev) => {
      const next = new Set([...prev].filter((abbr) => abbreviations.has(abbr)))
      return next.size === prev.size ? prev : next
    })
  }, [parts, setSoloedParts])
}
