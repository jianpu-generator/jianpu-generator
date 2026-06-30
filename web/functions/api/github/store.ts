import type { PagesFunction } from '@cloudflare/workers-types'
import { requireSession } from '../../_lib/auth'
import { loadGitHubStore } from '../../_lib/contents'
import { jsonResponse } from '../../_lib/response'
import type { Env } from '../../_lib/types'

export const onRequestGet: PagesFunction<Env> = async (context) => {
  const auth = await requireSession(context.request, context.env)
  if (!auth.ok) {
    return auth.response
  }

  try {
    const store = await loadGitHubStore(
      auth.session.accessToken,
      auth.session.username,
      auth.session.repo,
    )
    return jsonResponse(store)
  } catch (error) {
    const message =
      error instanceof Error ? error.message : 'Failed to load GitHub store'
    return jsonResponse({ error: message }, { status: 502 })
  }
}
