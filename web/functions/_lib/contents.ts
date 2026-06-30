import { GITHUB_USER_AGENT } from './github'
import {
  binPath,
  emptyManifest,
  type GitHubManifest,
  MANIFEST_PATH,
  parseManifest,
  scorePath,
  serializeManifest,
} from './manifest'

const SCORES_DIRECTORY = 'scores'
const BIN_DIRECTORY = 'bin'

interface GitHubContentFile {
  type: 'file'
  name: string
  path: string
  sha: string
  content?: string
  encoding?: string
}

interface GitHubContentDirectoryEntry {
  type: 'file' | 'dir' | 'submodule' | 'symlink'
  name: string
  path: string
}

interface PutFileResult {
  sha: string
}

export interface GitHubFileReadResult {
  content: string
  sha: string
}

export interface GitHubStoreResult {
  manifest: GitHubManifest
  manifestSha?: string
  scoreFiles: Record<string, string>
  binFiles: Record<string, string>
}

function githubHeaders(accessToken: string): HeadersInit {
  return {
    Accept: 'application/vnd.github+json',
    Authorization: `Bearer ${accessToken}`,
    'User-Agent': GITHUB_USER_AGENT,
  }
}

function contentsUrl(owner: string, repo: string, path: string): string {
  const encodedPath = path
    .split('/')
    .map((segment) => encodeURIComponent(segment))
    .join('/')
  return `https://api.github.com/repos/${owner}/${repo}/contents/${encodedPath}`
}

function decodeGitHubContent(base64Content: string): string {
  const normalized = base64Content.replace(/\n/g, '')
  const binary = atob(normalized)
  const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0))
  return new TextDecoder().decode(bytes)
}

function encodeGitHubContent(content: string): string {
  const bytes = new TextEncoder().encode(content)
  let binary = ''
  for (const byte of bytes) {
    binary += String.fromCharCode(byte)
  }
  return btoa(binary)
}

async function githubRequest(
  accessToken: string,
  url: string,
  init: RequestInit = {},
): Promise<Response> {
  return fetch(url, {
    ...init,
    headers: {
      ...githubHeaders(accessToken),
      ...init.headers,
    },
  })
}

async function readFileAtPath(
  accessToken: string,
  owner: string,
  repo: string,
  path: string,
): Promise<GitHubFileReadResult | null> {
  const response = await githubRequest(
    accessToken,
    contentsUrl(owner, repo, path),
  )

  if (response.status === 404) {
    return null
  }

  if (!response.ok) {
    const body = await response.text()
    throw new Error(`Failed to read ${path}: ${response.status} ${body}`)
  }

  const payload = (await response.json()) as GitHubContentFile
  if (payload.type !== 'file' || typeof payload.content !== 'string') {
    throw new Error(`Expected file at ${path}`)
  }

  return {
    content: decodeGitHubContent(payload.content),
    sha: payload.sha,
  }
}

async function listDirectoryFiles(
  accessToken: string,
  owner: string,
  repo: string,
  directory: string,
): Promise<string[]> {
  const response = await githubRequest(
    accessToken,
    contentsUrl(owner, repo, directory),
  )

  if (response.status === 404) {
    return []
  }

  if (!response.ok) {
    const body = await response.text()
    throw new Error(`Failed to list ${directory}: ${response.status} ${body}`)
  }

  const payload = (await response.json()) as
    | GitHubContentDirectoryEntry[]
    | GitHubContentFile

  if (!Array.isArray(payload)) {
    return []
  }

  return payload
    .filter((entry) => entry.type === 'file')
    .map((entry) => entry.name)
    .sort((left, right) => left.localeCompare(right))
}

async function readDirectoryFiles(
  accessToken: string,
  owner: string,
  repo: string,
  directory: string,
): Promise<Record<string, string>> {
  const fileNames = await listDirectoryFiles(
    accessToken,
    owner,
    repo,
    directory,
  )

  const entries = await Promise.all(
    fileNames.map(async (name) => {
      const file = await readFileAtPath(
        accessToken,
        owner,
        repo,
        `${directory}/${name}`,
      )
      return file ? ([name, file.content] as const) : null
    }),
  )

  return Object.fromEntries(
    entries.filter(
      (entry): entry is readonly [string, string] => entry !== null,
    ),
  )
}

async function putFileAtPath(
  accessToken: string,
  owner: string,
  repo: string,
  path: string,
  content: string,
  sha?: string,
): Promise<PutFileResult> {
  const body: Record<string, string> = {
    message: `Update ${path} via jianpu-generator`,
    content: encodeGitHubContent(content),
  }
  if (sha) {
    body.sha = sha
  }

  const response = await githubRequest(
    accessToken,
    contentsUrl(owner, repo, path),
    {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(body),
    },
  )

  if (response.status === 409) {
    const latest = await readFileAtPath(accessToken, owner, repo, path)
    if (!latest) {
      const bodyText = await response.text()
      throw new Error(`Conflict writing ${path}: ${bodyText}`)
    }

    return putFileAtPath(accessToken, owner, repo, path, content, latest.sha)
  }

  if (!response.ok) {
    const bodyText = await response.text()
    throw new Error(`Failed to write ${path}: ${response.status} ${bodyText}`)
  }

  const payload = (await response.json()) as {
    content?: { sha?: string }
  }
  const nextSha = payload.content?.sha
  if (!nextSha) {
    throw new Error(`Missing SHA after writing ${path}`)
  }

  return { sha: nextSha }
}

async function resolveExistingSha(
  accessToken: string,
  owner: string,
  repo: string,
  path: string,
  providedSha?: string,
): Promise<string | undefined> {
  if (providedSha) {
    return providedSha
  }

  const existing = await readFileAtPath(accessToken, owner, repo, path)
  return existing?.sha
}

export async function loadGitHubStore(
  accessToken: string,
  owner: string,
  repo: string,
): Promise<GitHubStoreResult> {
  const manifestFile = await readFileAtPath(
    accessToken,
    owner,
    repo,
    MANIFEST_PATH,
  )

  const manifest = manifestFile
    ? (parseManifest(manifestFile.content) ?? emptyManifest())
    : emptyManifest()

  const [scoreFiles, binFiles] = await Promise.all([
    readDirectoryFiles(accessToken, owner, repo, SCORES_DIRECTORY),
    readDirectoryFiles(accessToken, owner, repo, BIN_DIRECTORY),
  ])

  return {
    manifest,
    manifestSha: manifestFile?.sha,
    scoreFiles,
    binFiles,
  }
}

export async function readGitHubFile(
  accessToken: string,
  owner: string,
  repo: string,
  path: string,
): Promise<GitHubFileReadResult | null> {
  return readFileAtPath(accessToken, owner, repo, path)
}

export async function writeGitHubFile(
  accessToken: string,
  owner: string,
  repo: string,
  path: string,
  content: string,
  sha?: string,
): Promise<PutFileResult> {
  const resolvedSha = await resolveExistingSha(
    accessToken,
    owner,
    repo,
    path,
    sha,
  )
  return putFileAtPath(accessToken, owner, repo, path, content, resolvedSha)
}

export async function writeGitHubManifest(
  accessToken: string,
  owner: string,
  repo: string,
  manifest: GitHubManifest,
  sha?: string,
): Promise<PutFileResult> {
  const resolvedSha = await resolveExistingSha(
    accessToken,
    owner,
    repo,
    MANIFEST_PATH,
    sha,
  )
  return putFileAtPath(
    accessToken,
    owner,
    repo,
    MANIFEST_PATH,
    serializeManifest(manifest),
    resolvedSha,
  )
}

export function isAllowedRepositoryFilePath(path: string): boolean {
  if (!path || path.includes('..')) {
    return false
  }

  if (path.startsWith(`${SCORES_DIRECTORY}/`)) {
    return path.length > `${SCORES_DIRECTORY}/`.length
  }

  if (path.startsWith(`${BIN_DIRECTORY}/`)) {
    return path.length > `${BIN_DIRECTORY}/`.length
  }

  return false
}

export function joinPathParam(
  path: string | string[] | undefined,
): string | null {
  if (!path) {
    return null
  }

  if (Array.isArray(path)) {
    return path.join('/')
  }

  return path
}

export { binPath, scorePath }
