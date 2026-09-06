import { applyWrite, type StoredDoc, toPublicDoc } from './doc'
import type { SyncedWriteRequest } from './protocol'

export interface Env {
  SHARES: KVNamespace
}

// e.g. `/shares/AbCdEfGhIjK` — see `SHARE_ID_PATTERN` in
// `web/src/syncedShareUrl.ts` for the id shape this must accept.
const SHARE_PATH = /^\/shares\/([0-9A-Za-z_-]+)$/

// The web app calls this worker cross-origin (a different host than the
// site itself), so every response — including the preflight this triggers
// for `POST`'s JSON body — needs these. `*` is fine: there's no cookie/
// session auth here, just an unguessable share id plus a bearer-style
// `ownerToken` in the request body, neither of which `Access-Control-
// Allow-Origin` exposes to a origin that doesn't already have them.
const CORS_HEADERS = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type',
}

function withCors(response: Response): Response {
  const headers = new Headers(response.headers)
  for (const [name, value] of Object.entries(CORS_HEADERS)) {
    headers.set(name, value)
  }
  return new Response(response.body, { status: response.status, headers })
}

function shareKey(shareId: string): string {
  return `share:${shareId}`
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method === 'OPTIONS') {
      return new Response(null, { status: 204, headers: CORS_HEADERS })
    }

    const url = new URL(request.url)
    const match = SHARE_PATH.exec(url.pathname)
    if (!match) return withCors(new Response('Not Found', { status: 404 }))
    const key = shareKey(match[1])

    if (request.method === 'GET') {
      const stored = await env.SHARES.get<StoredDoc>(key, 'json')
      return withCors(Response.json(toPublicDoc(stored)))
    }

    if (request.method === 'POST') {
      let body: SyncedWriteRequest
      try {
        body = await request.json()
      } catch {
        return withCors(new Response('Bad Request', { status: 400 }))
      }
      const existing = await env.SHARES.get<StoredDoc>(key, 'json')
      const result = applyWrite(existing, body)
      if (result === 'forbidden') {
        return withCors(new Response('Forbidden', { status: 403 }))
      }
      await env.SHARES.put(key, JSON.stringify(result))
      return withCors(new Response(null, { status: 204 }))
    }

    return withCors(new Response('Method Not Allowed', { status: 405 }))
  },
} satisfies ExportedHandler<Env>
