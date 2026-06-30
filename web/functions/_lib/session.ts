import {
  clearCookie,
  parseCookies,
  SESSION_COOKIE_NAME,
  sessionCookie,
} from './cookies'
import { decryptPayload, encryptPayload } from './crypto'
import { requireEnv } from './env'
import type { Env, SessionData, SessionResponse } from './types'

export async function readSession(
  request: Request,
  env: Env,
): Promise<SessionData | null> {
  const cookies = parseCookies(request.headers.get('Cookie'))
  const encrypted = cookies[SESSION_COOKIE_NAME]
  if (!encrypted) {
    return null
  }

  const secret = requireEnv(env, 'SESSION_SECRET')
  const plaintext = await decryptPayload(secret, encrypted)
  if (!plaintext) {
    return null
  }

  try {
    const session = JSON.parse(plaintext) as SessionData
    if (
      typeof session.accessToken !== 'string' ||
      typeof session.username !== 'string' ||
      typeof session.repo !== 'string'
    ) {
      return null
    }
    return session
  } catch {
    return null
  }
}

export async function writeSessionCookie(
  session: SessionData,
  requestUrl: URL,
  env: Env,
): Promise<string> {
  const secret = requireEnv(env, 'SESSION_SECRET')
  const encrypted = await encryptPayload(secret, JSON.stringify(session))
  return sessionCookie(encrypted, requestUrl)
}

export function clearSessionCookie(requestUrl: URL): string {
  return clearCookie(SESSION_COOKIE_NAME, requestUrl)
}

export function toSessionResponse(
  session: SessionData | null,
): SessionResponse {
  if (!session) {
    return { connected: false }
  }

  return {
    connected: true,
    username: session.username,
    repo: session.repo,
  }
}
