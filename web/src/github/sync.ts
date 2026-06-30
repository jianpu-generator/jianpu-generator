import { DEMO_FILE_NAME, type FileStoreState } from '../fileStore'
import {
  binPath,
  fileStoreToManifest,
  type GitHubManifest,
  scorePath,
} from './manifest'

export interface FilePut {
  path: string
  content: string
}

export interface SyncPlan {
  filePuts: FilePut[]
  manifest: GitHubManifest | null
}

function manifestsEqual(left: GitHubManifest, right: GitHubManifest): boolean {
  return JSON.stringify(left) === JSON.stringify(right)
}

function collectFilePuts(
  baseline: FileStoreState,
  current: FileStoreState,
): FilePut[] {
  const puts: FilePut[] = []

  for (const [name, content] of Object.entries(current.userFiles)) {
    if (name === DEMO_FILE_NAME) continue
    if (baseline.userFiles[name] !== content) {
      puts.push({ path: scorePath(name), content })
    }
  }

  for (const [name, content] of Object.entries(current.bin)) {
    if (name === DEMO_FILE_NAME) continue
    if (baseline.bin[name] !== content) {
      puts.push({ path: binPath(name), content })
    }
  }

  return puts
}

export function buildSyncPlan(
  baseline: FileStoreState,
  current: FileStoreState,
): SyncPlan {
  const filePuts = collectFilePuts(baseline, current)
  const currentManifest = fileStoreToManifest(current)
  const baselineManifest = fileStoreToManifest(baseline)
  const manifest = manifestsEqual(baselineManifest, currentManifest)
    ? null
    : currentManifest

  return { filePuts, manifest }
}

export function syncPlanIsEmpty(plan: SyncPlan): boolean {
  return plan.filePuts.length === 0 && plan.manifest === null
}

export async function executeSyncPlan(plan: SyncPlan): Promise<void> {
  for (const { path, content } of plan.filePuts) {
    const response = await fetch(
      `/api/github/files/${path.split('/').map(encodeURIComponent).join('/')}`,
      {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content }),
      },
    )
    if (!response.ok) {
      const payload = (await response.json().catch(() => null)) as {
        error?: string
      } | null
      throw new Error(payload?.error ?? `Failed to save ${path}`)
    }
  }

  if (plan.manifest) {
    const response = await fetch('/api/github/manifest', {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(plan.manifest),
    })
    if (!response.ok) {
      const payload = (await response.json().catch(() => null)) as {
        error?: string
      } | null
      throw new Error(payload?.error ?? 'Failed to update manifest')
    }
  }
}
