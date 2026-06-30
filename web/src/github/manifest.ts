import { DEMO_FILE_NAME, type FileStoreState } from '../fileStore'

export interface GitHubManifest {
  active: string
  fileIds: Record<string, string>
  /** Binned file names only; content lives under `bin/`. */
  bin: string[]
}

export function scorePath(name: string): string {
  return `scores/${name}`
}

export function binPath(name: string): string {
  return `bin/${name}`
}

function isDemoFile(name: string): boolean {
  return name === DEMO_FILE_NAME
}

function sortedUserFileNames(userFiles: Record<string, string>): string[] {
  return Object.keys(userFiles)
    .filter((name) => !isDemoFile(name))
    .sort((a, b) => a.localeCompare(b))
}

function sortedBinNames(bin: Record<string, string>): string[] {
  return Object.keys(bin)
    .filter((name) => !isDemoFile(name))
    .sort((a, b) => a.localeCompare(b))
}

function ensureFileIds(
  userFiles: Record<string, string>,
  bin: Record<string, string>,
  existing: Record<string, string> | undefined,
): Record<string, string> {
  const fileIds = { ...existing }
  for (const name of Object.keys(userFiles)) {
    if (!fileIds[name]) fileIds[name] = crypto.randomUUID()
  }
  for (const name of Object.keys(bin)) {
    if (!fileIds[name]) fileIds[name] = crypto.randomUUID()
  }
  return fileIds
}

function userFilesWithoutDemo(
  userFiles: Record<string, string>,
): Record<string, string> {
  return Object.fromEntries(
    Object.entries(userFiles).filter(([name]) => !isDemoFile(name)),
  )
}

function binWithoutDemo(bin: Record<string, string>): Record<string, string> {
  return Object.fromEntries(
    Object.entries(bin).filter(([name]) => !isDemoFile(name)),
  )
}

function manifestActiveFromState(
  state: FileStoreState,
  userNames: string[],
): string {
  if (!isDemoFile(state.active) && userNames.includes(state.active)) {
    return state.active
  }
  return userNames[0] ?? ''
}

function fileStoreActiveFromManifest(
  manifestActive: string,
  userNames: string[],
): string {
  if (manifestActive && userNames.includes(manifestActive)) {
    return manifestActive
  }
  return userNames[0] ?? DEMO_FILE_NAME
}

export function fileStoreToManifest(state: FileStoreState): GitHubManifest {
  const userNames = sortedUserFileNames(state.userFiles)
  const binNames = sortedBinNames(state.bin)

  const fileIds: Record<string, string> = {}
  for (const name of [...userNames, ...binNames]) {
    const id = state.fileIds[name]
    if (id) fileIds[name] = id
  }

  return {
    active: manifestActiveFromState(state, userNames),
    fileIds,
    bin: binNames,
  }
}

export function manifestAndFilesToFileStore(
  manifest: GitHubManifest,
  scoreFiles: Record<string, string>,
  binFiles: Record<string, string>,
): FileStoreState {
  const userFiles = userFilesWithoutDemo(scoreFiles)
  const bin = binWithoutDemo(binFiles)
  const userNames = sortedUserFileNames(userFiles)

  return {
    active: fileStoreActiveFromManifest(manifest.active, userNames),
    userFiles,
    bin,
    fileIds: ensureFileIds(userFiles, bin, manifest.fileIds),
  }
}
