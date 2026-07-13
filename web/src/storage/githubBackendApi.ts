import type { Octokit } from '@octokit/rest'
import { decodeBase64, encodeBase64, statusOf } from './githubBackendUtils'

export interface GithubContentsApiConfig {
  octokit: Octokit
  owner: string
  repo: string
  branch?: string
}

/** Low-level wrappers around GitHub's Contents API used by `createGithubBackend`: fetching
 * a path's current `sha`/content, listing `.jianpu` files in a directory, and the
 * create-only/fetch-sha-then-write/delete write operations. */
export function createGithubContentsApi({
  octokit,
  owner,
  repo,
  branch,
}: GithubContentsApiConfig) {
  async function fetchSha(path: string): Promise<string | undefined> {
    try {
      const { data } = await octokit.rest.repos.getContent({
        owner,
        repo,
        path,
        ref: branch,
      })
      return !Array.isArray(data) && 'sha' in data ? data.sha : undefined
    } catch (error) {
      if (statusOf(error) === 404) return undefined
      throw error
    }
  }

  async function fetchFileContent(path: string): Promise<string> {
    const { data } = await octokit.rest.repos.getContent({
      owner,
      repo,
      path,
      ref: branch,
    })
    if (Array.isArray(data) || data.type !== 'file' || !data.content) {
      throw new Error(`githubBackend: expected a file at ${path}`)
    }
    return decodeBase64(data.content)
  }

  async function listJianpuFiles(
    dirPath: string,
  ): Promise<Record<string, string>> {
    let entries: { name: string; path: string; type: string }[]
    try {
      const { data } = await octokit.rest.repos.getContent({
        owner,
        repo,
        path: dirPath,
        ref: branch,
      })
      entries = Array.isArray(data) ? data : []
    } catch (error) {
      if (statusOf(error) === 404) return {}
      throw error
    }

    const files = entries.filter(
      (entry) => entry.type === 'file' && entry.name.endsWith('.jianpu'),
    )
    const contents = await Promise.all(
      files.map((file) => fetchFileContent(file.path)),
    )
    const result: Record<string, string> = {}
    files.forEach((file, index) => {
      result[file.name] = contents[index] ?? ''
    })
    return result
  }

  /** Create-only write, no sha lookup — for paths guaranteed not to exist
   * yet (new file, rename/restore destination, duplicate destination). */
  async function createOnly(
    path: string,
    content: string,
    message: string,
  ): Promise<void> {
    await octokit.rest.repos.createOrUpdateFileContents({
      owner,
      repo,
      path,
      message,
      content: encodeBase64(content),
      branch,
    })
  }

  /** Fetch-sha-then-write, for paths that may already exist (active-file
   * saves, and the `trash/` destination of a delete — which can already hold
   * a stale entry from an earlier restore-then-delete cycle). */
  async function putFile(
    path: string,
    content: string,
    message: string,
  ): Promise<void> {
    const sha = await fetchSha(path)
    await octokit.rest.repos.createOrUpdateFileContents({
      owner,
      repo,
      path,
      message,
      content: encodeBase64(content),
      sha,
      branch,
    })
  }

  async function deleteFileAt(path: string, message: string): Promise<void> {
    const sha = await fetchSha(path)
    if (!sha) return
    await octokit.rest.repos.deleteFile({
      owner,
      repo,
      path,
      message,
      sha,
      branch,
    })
  }

  return {
    fetchSha,
    fetchFileContent,
    listJianpuFiles,
    createOnly,
    putFile,
    deleteFileAt,
  }
}
