import type { PagesFunction } from '@cloudflare/workers-types'
import { jsonResponse } from '../../_lib/response'
import { clearSessionCookie } from '../../_lib/session'
import type { Env } from '../../_lib/types'

export const onRequestPost: PagesFunction<Env> = async (context) => {
  const requestUrl = new URL(context.request.url)
  return jsonResponse(
    { ok: true },
    {
      headers: {
        'Set-Cookie': clearSessionCookie(requestUrl),
      },
    },
  )
}
