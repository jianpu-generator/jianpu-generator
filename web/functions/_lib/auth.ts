import { jsonResponse } from './response'
import { readSession } from './session'
import type { Env, SessionData } from './types'

export type SessionAuthResult =
  | { ok: true; session: SessionData }
  | { ok: false; response: Response }

export async function requireSession(
  request: Request,
  env: Env,
): Promise<SessionAuthResult> {
  const session = await readSession(request, env)
  if (!session) {
    return {
      ok: false,
      response: jsonResponse({ error: 'unauthorized' }, { status: 401 }),
    }
  }

  return { ok: true, session }
}
