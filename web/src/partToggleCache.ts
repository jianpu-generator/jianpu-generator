export const LEGACY_PART_TOGGLES_KEY = 'jianpu:part-toggles:v1'

export type PartToggleWorkspace = 'local' | 'github'

export interface PartToggleState {
  disabledParts: string[]
  disabledLyrics: string[]
  soloedParts: string[]
}

type PartToggleCache = Record<string, PartToggleState>

function partTogglesStorageKey(workspace: PartToggleWorkspace): string {
  return workspace === 'local'
    ? 'jianpu:part-toggles:local'
    : 'jianpu:part-toggles:github'
}

function readCache(workspace: PartToggleWorkspace): PartToggleCache {
  try {
    const key = partTogglesStorageKey(workspace)
    let raw = localStorage.getItem(key)
    if (raw == null && workspace === 'local') {
      raw = localStorage.getItem(LEGACY_PART_TOGGLES_KEY)
    }
    if (raw != null) {
      const parsed = JSON.parse(raw) as PartToggleCache
      if (parsed && typeof parsed === 'object') return parsed
    }
  } catch {
    // ignore corrupt storage
  }
  return {}
}

export function readPartTogglesForFile(
  fileId: string,
  workspace: PartToggleWorkspace = 'local',
): PartToggleState | null {
  const entry = readCache(workspace)[fileId]
  if (entry == null) return null
  return {
    disabledParts: entry.disabledParts ?? [],
    disabledLyrics: entry.disabledLyrics ?? [],
    soloedParts: entry.soloedParts ?? [],
  }
}

export function writePartTogglesForFile(
  fileId: string,
  state: PartToggleState,
  workspace: PartToggleWorkspace = 'local',
): void {
  try {
    const cache = readCache(workspace)
    cache[fileId] = state
    localStorage.setItem(
      partTogglesStorageKey(workspace),
      JSON.stringify(cache),
    )
  } catch {
    // ignore quota errors
  }
}
