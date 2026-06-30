import type { PagesFunction } from '@cloudflare/workers-types'
import { jsonResponse } from '../../_lib/response'
import { readSession, toSessionResponse } from '../../_lib/session'
import type { Env } from '../../_lib/types'

export const onRequestGet: PagesFunction<Env> = async (context) => {
  const session = await readSession(context.request, context.env)
  return jsonResponse(toSessionResponse(session))
}
