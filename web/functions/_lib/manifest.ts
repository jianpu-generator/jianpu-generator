export interface GitHubManifest {
  active: string
  fileIds: Record<string, string>
  /** Binned file names only; content lives under `bin/`. */
  bin: string[]
}

export const MANIFEST_PATH = '.jianpu/manifest.json'

export function scorePath(name: string): string {
  return `scores/${name}`
}

export function binPath(name: string): string {
  return `bin/${name}`
}

export function emptyManifest(): GitHubManifest {
  return {
    active: '',
    fileIds: {},
    bin: [],
  }
}

export function parseManifest(raw: string): GitHubManifest | null {
  try {
    const parsed = JSON.parse(raw) as Partial<GitHubManifest>
    if (
      typeof parsed.active !== 'string' ||
      typeof parsed.fileIds !== 'object' ||
      parsed.fileIds === null ||
      !Array.isArray(parsed.bin) ||
      !parsed.bin.every((name) => typeof name === 'string')
    ) {
      return null
    }

    const fileIds: Record<string, string> = {}
    for (const [name, id] of Object.entries(parsed.fileIds)) {
      if (typeof id === 'string') {
        fileIds[name] = id
      }
    }

    return {
      active: parsed.active,
      fileIds,
      bin: [...parsed.bin],
    }
  } catch {
    return null
  }
}

export function serializeManifest(manifest: GitHubManifest): string {
  return `${JSON.stringify(manifest, null, 2)}\n`
}

export function isValidManifest(manifest: unknown): manifest is GitHubManifest {
  if (typeof manifest !== 'object' || manifest === null) {
    return false
  }

  const candidate = manifest as Partial<GitHubManifest>
  if (
    typeof candidate.active !== 'string' ||
    typeof candidate.fileIds !== 'object' ||
    candidate.fileIds === null ||
    !Array.isArray(candidate.bin)
  ) {
    return false
  }

  for (const id of Object.values(candidate.fileIds)) {
    if (typeof id !== 'string') {
      return false
    }
  }

  return candidate.bin.every((name) => typeof name === 'string')
}
