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
  const [soloedParts, setSoloedParts] = useState<Set<string>>(() => {
    const cached = readPartTogglesForFile(fileId)
    return new Set(cached?.soloedParts ?? [])
  })
  const skipToggleSaveRef = useRef(false)

  useEffect(() => {
    skipToggleSaveRef.current = true
    const cached = readPartTogglesForFile(fileId)
    setDisabledParts(new Set(cached?.disabledParts ?? []))
    setSoloedParts(new Set(cached?.soloedParts ?? []))
  }, [fileId])

  useEffect(() => {
    if (skipToggleSaveRef.current) {
      skipToggleSaveRef.current = false
      return
    }
    writePartTogglesForFile(fileId, {
      disabledParts: [...disabledParts],
      soloedParts: [...soloedParts],
    })
  }, [fileId, disabledParts, soloedParts])

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

  /** The raw toggle state as handed to Rust, which decides what it means. */
  const partToggles = useMemo<PartToggleState>(
    () => ({
      hiddenParts: [...disabledParts],
      soloedParts: [...soloedParts],
    }),
    [disabledParts, soloedParts],
  )

  return {
    partToggles,
    disabledParts,
    setDisabledParts,
    soloedParts,
    setSoloedParts,
    handlePartToggle,
    handleSoloToggle,
  }
}
