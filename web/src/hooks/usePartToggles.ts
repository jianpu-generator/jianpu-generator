import { useCallback, useEffect, useRef, useState } from 'react'
import {
  readPartTogglesForFile,
  writePartTogglesForFile,
} from '../partToggleCache'
import type { PartInfo } from '../types'

/** True when every part is either soloed-out or individually disabled, leaving nothing audible/visible. */
export function noPartsSelected(
  parts: PartInfo[],
  disabledParts: ReadonlySet<string>,
  soloedParts: ReadonlySet<string>,
): boolean {
  return (
    parts.length > 0 &&
    soloedParts.size === 0 &&
    parts.every((part) => disabledParts.has(part.abbreviation))
  )
}

export function usePartToggles(fileId: string) {
  const [disabledParts, setDisabledParts] = useState<Set<string>>(() => {
    const cached = readPartTogglesForFile(fileId)
    return new Set(cached?.disabledParts ?? [])
  })
  const [disabledLyrics, setDisabledLyrics] = useState<Set<string>>(() => {
    const cached = readPartTogglesForFile(fileId)
    return new Set(cached?.disabledLyrics ?? [])
  })
  const [soloedParts, setSoloedParts] = useState<Set<string>>(() => {
    const cached = readPartTogglesForFile(fileId)
    return new Set(cached?.soloedParts ?? [])
  })
  const skipToggleSaveRef = useRef(false)

  useEffect(() => {
    skipToggleSaveRef.current = true
    const cached = readPartTogglesForFile(fileId)
    setDisabledParts(new Set(cached?.disabledParts ?? []))
    setDisabledLyrics(new Set(cached?.disabledLyrics ?? []))
    setSoloedParts(new Set(cached?.soloedParts ?? []))
  }, [fileId])

  useEffect(() => {
    if (skipToggleSaveRef.current) {
      skipToggleSaveRef.current = false
      return
    }
    writePartTogglesForFile(fileId, {
      disabledParts: [...disabledParts],
      disabledLyrics: [...disabledLyrics],
      soloedParts: [...soloedParts],
    })
  }, [fileId, disabledParts, disabledLyrics, soloedParts])

  const handlePartToggle = useCallback(
    (abbreviation: string, enabled: boolean) => {
      setDisabledParts((prev) => {
        const next = new Set(prev)
        if (enabled) {
          next.delete(abbreviation)
        } else {
          next.add(abbreviation)
        }
        return next
      })
    },
    [],
  )

  const handleLyricsToggle = useCallback(
    (abbreviation: string, enabled: boolean) => {
      setDisabledLyrics((prev) => {
        const next = new Set(prev)
        if (enabled) {
          next.delete(abbreviation)
        } else {
          next.add(abbreviation)
        }
        return next
      })
    },
    [],
  )

  const handleSoloToggle = useCallback(
    (abbreviation: string, soloed: boolean) => {
      setSoloedParts((prev) => {
        const next = new Set(prev)
        if (soloed) {
          next.add(abbreviation)
        } else {
          next.delete(abbreviation)
        }
        return next
      })
    },
    [],
  )

  return {
    disabledParts,
    setDisabledParts,
    disabledLyrics,
    setDisabledLyrics,
    soloedParts,
    setSoloedParts,
    handlePartToggle,
    handleLyricsToggle,
    handleSoloToggle,
  }
}
