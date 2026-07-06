import NEW_FILE_TEMPLATE from '../../new_file_template.jianpu?raw'
import { DEFAULT_SOURCE, DEMO_FILE_NAME } from './defaultSource'
import {
  clearShareHash,
  parseShareFromHash,
  type SharePayload,
} from './shareUrl'

export { DEMO_FILE_NAME }

export const DEMO_FILE_ID = 'jianpu:reference'

export const FILE_STORE_KEY = 'jianpu:files:v1'
const STORAGE_KEY = 'jianpu:source:v5'

const DEFAULT_FILE_STORE: FileStoreState = {
  active: DEMO_FILE_NAME,
  userFiles: {},
  bin: {},
  fileIds: {},
}

function generateFileId(): string {
  return crypto.randomUUID()
}

export interface FileStoreState {
  active: string
  userFiles: Record<string, string>
  bin: Record<string, string>
  /** Stable ID per file name (active and binned); survives renames. */
  fileIds: Record<string, string>
}

export function fileIdForName(state: FileStoreState, name: string): string {
  if (isDemoFile(name)) return DEMO_FILE_ID
  const id = state.fileIds[name]
  if (!id) throw new Error(`Missing file ID for ${name}`)
  return id
}

function isDemoFile(name: string): boolean {
  return name === DEMO_FILE_NAME
}

export function fileContent(state: FileStoreState, name: string): string {
  if (isDemoFile(name)) return DEFAULT_SOURCE
  return state.userFiles[name] ?? ''
}

export function sortedFileNames(state: FileStoreState): string[] {
  const names = new Set<string>([
    DEMO_FILE_NAME,
    ...Object.keys(state.userFiles),
  ])
  return [...names].sort((a, b) => a.localeCompare(b))
}

export function sortedBinNames(state: FileStoreState): string[] {
  return Object.keys(state.bin).sort((a, b) => a.localeCompare(b))
}

function reservedNames(state: FileStoreState): Set<string> {
  return new Set([
    DEMO_FILE_NAME,
    ...Object.keys(state.userFiles),
    ...Object.keys(state.bin),
  ])
}

function uniqueName(base: string, taken: Set<string>): string {
  if (!taken.has(base)) return base
  const dot = base.lastIndexOf('.')
  const stem = dot > 0 ? base.slice(0, dot) : base
  const ext = dot > 0 ? base.slice(dot) : '.jianpu'
  let n = 2
  while (taken.has(`${stem} ${n}${ext}`)) n++
  return `${stem} ${n}${ext}`
}

function sanitizeFileName(raw: string): string {
  const trimmed = raw.trim()
  if (!trimmed) return 'untitled.jianpu'
  return trimmed.endsWith('.jianpu') ? trimmed : `${trimmed}.jianpu`
}

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

export function updateActiveContent(
  state: FileStoreState,
  content: string,
): FileStoreState {
  if (isDemoFile(state.active)) return state
  return {
    ...state,
    userFiles: { ...state.userFiles, [state.active]: content },
  }
}

export function selectFile(
  state: FileStoreState,
  name: string,
): FileStoreState {
  const names = sortedFileNames(state)
  if (!names.includes(name)) return state
  return { ...state, active: name }
}

export function createFile(state: FileStoreState): FileStoreState {
  const taken = reservedNames(state)
  const name = uniqueName('untitled.jianpu', taken)
  return {
    ...state,
    active: name,
    userFiles: { ...state.userFiles, [name]: NEW_FILE_TEMPLATE },
    fileIds: { ...state.fileIds, [name]: generateFileId() },
  }
}

export function duplicateFile(state: FileStoreState): FileStoreState {
  const source = state.active
  const taken = reservedNames(state)
  const name = uniqueName(source, taken)
  const content = fileContent(state, source)
  return {
    ...state,
    active: name,
    userFiles: { ...state.userFiles, [name]: content },
    fileIds: { ...state.fileIds, [name]: generateFileId() },
  }
}

export function renameFile(
  state: FileStoreState,
  from: string,
  toRaw: string,
): FileStoreState {
  if (isDemoFile(from)) return state
  const to = sanitizeFileName(toRaw)
  if (to === from) return state
  const names = sortedFileNames(state)
  if (!names.includes(from) || isDemoFile(to)) return state
  if (reservedNames(state).has(to) && to !== from) return state

  const { [from]: content, ...rest } = state.userFiles
  if (content === undefined) return state

  const { [from]: fileId, ...restIds } = state.fileIds
  return {
    ...state,
    active: state.active === from ? to : state.active,
    userFiles: { ...rest, [to]: content },
    fileIds: { ...restIds, [to]: fileId ?? generateFileId() },
  }
}

export function deleteFile(
  state: FileStoreState,
  name: string,
): FileStoreState {
  if (isDemoFile(name)) return state
  const content = state.userFiles[name]
  if (content === undefined) return state

  const { [name]: _, ...rest } = state.userFiles
  const remaining = sortedFileNames({ ...state, userFiles: rest })
  const nextActive =
    state.active === name ? (remaining[0] ?? DEMO_FILE_NAME) : state.active

  return {
    ...state,
    active: nextActive,
    userFiles: rest,
    bin: { ...state.bin, [name]: content },
  }
}

export function restoreFile(
  state: FileStoreState,
  name: string,
): FileStoreState {
  const content = state.bin[name]
  if (content === undefined) return state

  const activeNames = new Set(sortedFileNames(state))
  const restoreName = activeNames.has(name)
    ? uniqueName(name, activeNames)
    : name

  const { [name]: _, ...restBin } = state.bin
  const fileIds =
    restoreName === name
      ? state.fileIds
      : {
          ...state.fileIds,
          [restoreName]: state.fileIds[name] ?? generateFileId(),
        }

  return {
    ...state,
    active: restoreName,
    userFiles: { ...state.userFiles, [restoreName]: content },
    bin: restBin,
    fileIds,
  }
}

function mergeAddedRemoved(
  prevRecord: Record<string, string>,
  baseRecord: Record<string, string>,
  nextRecord: Record<string, string>,
  resolveAdded: (key: string) => string,
): Record<string, string> {
  const result = { ...prevRecord }
  for (const key of Object.keys(baseRecord)) {
    if (!(key in nextRecord)) delete result[key]
  }
  for (const key of Object.keys(nextRecord)) {
    if (!(key in baseRecord)) result[key] = resolveAdded(key)
  }
  return result
}

/**
 * Reconciles the result of an async structural operation (create/duplicate/
 * rename/delete/restore) with whatever the store has become since the
 * operation's `base` snapshot was read — instead of blindly replacing state
 * with `next`, which would discard any edits or other operations applied to
 * `prev` while the async call was in flight. Only the keys the operation
 * actually added or removed (relative to `base`) are applied to `prev`;
 * every other key is left exactly as `prev` has it. For a moved key (e.g.
 * rename's `from` -> `to`), the destination's content is taken from `prev`'s
 * current value of the source key when still present, so a concurrent edit
 * to the file being renamed/restored isn't lost either.
 */
export function mergeBackendResult(
  prev: FileStoreState,
  base: FileStoreState,
  next: FileStoreState,
): FileStoreState {
  const removedFrom = (
    record: 'userFiles' | 'bin',
  ): string[] =>
    Object.keys(base[record]).filter((key) => !(key in next[record]))

  const findMovedSource = (content: string): string | undefined => {
    const fromUserFiles = removedFrom('userFiles').find(
      (key) => base.userFiles[key] === content,
    )
    if (fromUserFiles !== undefined) {
      return prev.userFiles[fromUserFiles] ?? prev.bin[fromUserFiles]
    }
    const fromBin = removedFrom('bin').find((key) => base.bin[key] === content)
    if (fromBin !== undefined) {
      return prev.userFiles[fromBin] ?? prev.bin[fromBin]
    }
    return undefined
  }

  return {
    active: next.active !== base.active ? next.active : prev.active,
    userFiles: mergeAddedRemoved(
      prev.userFiles,
      base.userFiles,
      next.userFiles,
      (key) => findMovedSource(next.userFiles[key]) ?? next.userFiles[key],
    ),
    bin: mergeAddedRemoved(
      prev.bin,
      base.bin,
      next.bin,
      (key) => findMovedSource(next.bin[key]) ?? next.bin[key],
    ),
    fileIds: mergeAddedRemoved(
      prev.fileIds,
      base.fileIds,
      next.fileIds,
      (key) => next.fileIds[key],
    ),
  }
}

export function isReadOnlyFile(name: string): boolean {
  return isDemoFile(name)
}

export function importSharedFile(
  state: FileStoreState,
  filenameRaw: string,
  content: string,
): FileStoreState {
  let name = sanitizeFileName(filenameRaw)
  if (isDemoFile(name)) {
    name = 'shared.jianpu'
  }

  const taken = reservedNames(state)
  name = uniqueName(name, taken)

  return {
    ...state,
    active: name,
    userFiles: { ...state.userFiles, [name]: content },
    fileIds: { ...state.fileIds, [name]: generateFileId() },
  }
}

let cachedSharePayload: SharePayload | null | undefined

function consumeSharePayload(): SharePayload | null {
  if (cachedSharePayload !== undefined) {
    return cachedSharePayload
  }

  const shared = parseShareFromHash()
  cachedSharePayload = shared
  if (shared) {
    clearShareHash()
  }
  return shared
}

export function applyShareIfPresent(state: FileStoreState): FileStoreState {
  const shared = consumeSharePayload()
  if (!shared) return state

  return importSharedFile(state, shared.filename, shared.content)
}
