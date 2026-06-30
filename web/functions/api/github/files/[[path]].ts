import type { PagesFunction } from '@cloudflare/workers-types'
import { requireSession } from '../../../_lib/auth'
import {
  isAllowedRepositoryFilePath,
  joinPathParam,
  readGitHubFile,
  writeGitHubFile,
} from '../../../_lib/contents'
import { jsonResponse } from '../../../_lib/response'
import type { Env } from '../../../_lib/types'

interface PutFileBody {
  content?: unknown
  sha?: unknown
}

export const onRequestGet: PagesFunction<Env> = async (context) => {
  const auth = await requireSession(context.request, context.env)
  if (!auth.ok) {
    return auth.response
  }

  const path = joinPathParam(context.params.path)
  if (!path || !isAllowedRepositoryFilePath(path)) {
    return jsonResponse({ error: 'invalid_path' }, { status: 400 })
  }

  try {
    const file = await readGitHubFile(
      auth.session.accessToken,
      auth.session.username,
      auth.session.repo,
      path,
    )

    if (!file) {
      return jsonResponse({ error: 'not_found' }, { status: 404 })
    }

    return jsonResponse(file)
  } catch (error) {
    const message =
      error instanceof Error ? error.message : 'Failed to read GitHub file'
    return jsonResponse({ error: message }, { status: 502 })
  }
}

export const onRequestPut: PagesFunction<Env> = async (context) => {
  const auth = await requireSession(context.request, context.env)
  if (!auth.ok) {
    return auth.response
  }

  const path = joinPathParam(context.params.path)
  if (!path || !isAllowedRepositoryFilePath(path)) {
    return jsonResponse({ error: 'invalid_path' }, { status: 400 })
  }

  let body: PutFileBody
  try {
    body = (await context.request.json()) as PutFileBody
  } catch {
    return jsonResponse({ error: 'invalid_json' }, { status: 400 })
  }

  if (typeof body.content !== 'string') {
    return jsonResponse({ error: 'content_required' }, { status: 400 })
  }

  const sha = typeof body.sha === 'string' ? body.sha : undefined

  try {
    const result = await writeGitHubFile(
      auth.session.accessToken,
      auth.session.username,
      auth.session.repo,
      path,
      body.content,
      sha,
    )
    return jsonResponse(result)
  } catch (error) {
    const message =
      error instanceof Error ? error.message : 'Failed to write GitHub file'
    return jsonResponse({ error: message }, { status: 502 })
  }
}
