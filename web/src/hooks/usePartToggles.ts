import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import {
  readPartTogglesForFile,
  writePartTogglesForFile,
} from '../partToggleCache'
import type { PartToggleState } from '../types'

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
  const [soloedLyrics, setSoloedLyrics] = useState<Set<string>>(() => {
    const cached = readPartTogglesForFile(fileId)
    return new Set(cached?.soloedLyrics ?? [])
  })
  const skipToggleSaveRef = useRef(false)

  useEffect(() => {
    skipToggleSaveRef.current = true
    const cached = readPartTogglesForFile(fileId)
    setDisabledParts(new Set(cached?.disabledParts ?? []))
    setDisabledLyrics(new Set(cached?.disabledLyrics ?? []))
    setSoloedParts(new Set(cached?.soloedParts ?? []))
    setSoloedLyrics(new Set(cached?.soloedLyrics ?? []))
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
      soloedLyrics: [...soloedLyrics],
    })
  }, [fileId, disabledParts, disabledLyrics, soloedParts, soloedLyrics])

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

  const handleLyricsSoloToggle = useCallback(
    (abbreviation: string, soloed: boolean) => {
      setSoloedLyrics((prev) => {
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

  /** The raw toggle state as handed to Rust, which decides what it means. */
  const partToggles = useMemo<PartToggleState>(
    () => ({
      hiddenNotes: [...disabledParts],
      soloedNotes: [...soloedParts],
      hiddenLyrics: [...disabledLyrics],
      soloedLyrics: [...soloedLyrics],
    }),
    [disabledParts, soloedParts, disabledLyrics, soloedLyrics],
  )

  return {
    partToggles,
    disabledParts,
    setDisabledParts,
    disabledLyrics,
    setDisabledLyrics,
    soloedParts,
    setSoloedParts,
    soloedLyrics,
    setSoloedLyrics,
    handlePartToggle,
    handleLyricsToggle,
    handleSoloToggle,
    handleLyricsSoloToggle,
  }
}
