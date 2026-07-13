import { DEMO_FILE_NAME } from './defaultSource'
import {
  DEFAULT_FILE_STORE,
  FILE_STORE_KEY,
  type FileStoreState,
  generateFileId,
  STORAGE_KEY,
  sortedFileNames,
} from './fileStore'

function ensureFileIds(
  userFiles: Record<string, string>,
  bin: Record<string, string>,
  existing: Record<string, string> | undefined,
): Record<string, string> {
  const fileIds = { ...existing }
  for (const name of Object.keys(userFiles)) {
    if (!fileIds[name]) fileIds[name] = generateFileId()
  }
  for (const name of Object.keys(bin)) {
    if (!fileIds[name]) fileIds[name] = generateFileId()
  }
  return fileIds
}

function normalizeState(parsed: Partial<FileStoreState>): FileStoreState {
  const userFiles = { ...parsed.userFiles }
  delete userFiles[DEMO_FILE_NAME]
  const bin = { ...parsed.bin }
  delete bin[DEMO_FILE_NAME]

  const state: FileStoreState = {
    active: parsed.active ?? DEMO_FILE_NAME,
    userFiles,
    bin,
    fileIds: ensureFileIds(userFiles, bin, parsed.fileIds),
  }
  const names = sortedFileNames(state)
  return {
    ...state,
    active: names.includes(state.active) ? state.active : DEMO_FILE_NAME,
  }
}

function fileIdsNeedMigration(
  stored: Partial<FileStoreState>,
  normalized: FileStoreState,
): boolean {
  if (!stored.fileIds) return true
  for (const name of [
    ...Object.keys(normalized.userFiles),
    ...Object.keys(normalized.bin),
  ]) {
    if (!stored.fileIds[name]) return true
  }
  return false
}

function persistFileStoreMigration(
  raw: string,
  normalized: FileStoreState,
): void {
  try {
    const stored = JSON.parse(raw) as Partial<FileStoreState>
    if (fileIdsNeedMigration(stored, normalized)) {
      localStorage.setItem(FILE_STORE_KEY, JSON.stringify(normalized))
    }
  } catch {
    // ignore migration write failures
  }
}

function parseStoredFileStore(raw: string): FileStoreState | null {
  try {
    const parsed = JSON.parse(raw) as Partial<FileStoreState>
    if (parsed && typeof parsed.active === 'string' && parsed.userFiles) {
      return normalizeState({
        ...parsed,
        bin: parsed.bin ?? {},
      })
    }
  } catch {
    // ignore corrupt storage
  }
  return null
}

function readLegacyFileStore(): FileStoreState | null {
  try {
    const legacy = localStorage.getItem(STORAGE_KEY)
    if (legacy != null) {
      const userFiles = { 'untitled.jianpu': legacy }
      return {
        active: 'untitled.jianpu',
        userFiles,
        bin: {},
        fileIds: ensureFileIds(userFiles, {}, undefined),
      }
    }
  } catch {
    // ignore
  }
  return null
}

export function readInitialFileStore(): FileStoreState {
  try {
    const raw = localStorage.getItem(FILE_STORE_KEY)
    if (raw != null) {
      const parsed = parseStoredFileStore(raw)
      if (parsed) {
        persistFileStoreMigration(raw, parsed)
        return parsed
      }
    }
  } catch {
    // ignore
  }

  return readLegacyFileStore() ?? DEFAULT_FILE_STORE
}

export function deserializeFileStore(raw: string): FileStoreState {
  const parsed = parseStoredFileStore(raw)
  if (parsed) {
    persistFileStoreMigration(raw, parsed)
    return parsed
  }
  return readLegacyFileStore() ?? DEFAULT_FILE_STORE
}
