import type { PagesFunction } from '@cloudflare/workers-types'
import { requireSession } from '../../_lib/auth'
import { writeGitHubManifest } from '../../_lib/contents'
import { type GitHubManifest, isValidManifest } from '../../_lib/manifest'
import { jsonResponse } from '../../_lib/response'
import type { Env } from '../../_lib/types'

interface PatchManifestBody extends GitHubManifest {
  sha?: unknown
}

export const onRequestPatch: PagesFunction<Env> = async (context) => {
  const auth = await requireSession(context.request, context.env)
  if (!auth.ok) {
    return auth.response
  }

  let body: PatchManifestBody
  try {
    body = (await context.request.json()) as PatchManifestBody
  } catch {
    return jsonResponse({ error: 'invalid_json' }, { status: 400 })
  }

  const { sha: rawSha, ...manifestCandidate } = body
  if (!isValidManifest(manifestCandidate)) {
    return jsonResponse({ error: 'invalid_manifest' }, { status: 400 })
  }

  const sha = typeof rawSha === 'string' ? rawSha : undefined

  try {
    const result = await writeGitHubManifest(
      auth.session.accessToken,
      auth.session.username,
      auth.session.repo,
      manifestCandidate,
      sha,
    )
    return jsonResponse(result)
  } catch (error) {
    const message =
      error instanceof Error ? error.message : 'Failed to update manifest'
    return jsonResponse({ error: message }, { status: 502 })
  }
}
