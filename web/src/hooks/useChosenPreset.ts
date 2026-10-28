import { useCallback, useEffect, useState } from 'react'

export const CHOSEN_PRESET_KEY = 'jianpu:chosen-preset:v1'

/** File id → the preset chosen in the preset menu. Default has no entry. */
type ChosenPresetCache = Record<string, string>

function readCache(): ChosenPresetCache {
  try {
    const raw = localStorage.getItem(CHOSEN_PRESET_KEY)
    if (raw != null) {
      const parsed = JSON.parse(raw) as ChosenPresetCache
      if (parsed && typeof parsed === 'object') return parsed
    }
  } catch {
    // ignore corrupt storage
  }
  return {}
}

function writeCache(fileId: string, presetName: string | null): void {
  try {
    const cache = readCache()
    if (presetName == null) delete cache[fileId]
    else cache[fileId] = presetName
    localStorage.setItem(CHOSEN_PRESET_KEY, JSON.stringify(cache))
  } catch {
    // ignore quota errors
  }
}

/** The preset chosen for `fileId`, remembered per browser like the part
 * toggles. `null` is Default. A name the file no longer has is kept; Rust
 * resolves it as Default (`resolvePartVisibility`'s `presetName`). */
export function useChosenPreset(fileId: string) {
  const [chosenPreset, setChosenPresetState] = useState<string | null>(
    () => readCache()[fileId] ?? null,
  )

  useEffect(() => {
    setChosenPresetState(readCache()[fileId] ?? null)
  }, [fileId])

  const setChosenPreset = useCallback(
    (presetName: string | null) => {
      setChosenPresetState(presetName)
      writeCache(fileId, presetName)
    },
    [fileId],
  )

  return { chosenPreset, setChosenPreset }
}
