import { beforeEach, vi } from 'vitest'
import type { ApiError } from '../syncedShare/workerClient'

/** Shared `fetch` mock + response/request helpers for `cloudBackend.test.ts`
 * and `cloudBackendRetry.test.ts` -- split out so each test file stays
 * under this repo's 400-line cap while keeping identical setup. Vitest
 * isolates each test file's module graph by default, so the `fetchMock`
 * instance below is fresh per test file despite the shared import. The
 * worker client (openapi-fetch) calls `fetch` with a single `Request`. */
export const fetchMock = vi.fn<(request: Request) => Promise<Response>>()

export function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

/** A worker failure response, its body checked against the generated
 * `ApiError` union. */
export function apiErrorResponse(status: number, error: ApiError): Response {
  return jsonResponse(status, error)
}

export function emptyResponse(status: number): Response {
  return new Response(null, { status })
}

beforeEach(() => {
  fetchMock.mockReset()
  vi.stubGlobal('fetch', fetchMock)
})

export const config = { token: 'test-token', workerHost: 'localhost:8787' }

export interface RecordedCall {
  url: string
  method: string
  body: Record<string, unknown>
}

/** The `index`-th (negative counts from the end) request `fetch` got. */
export async function callAt(index: number): Promise<RecordedCall> {
  const request = fetchMock.mock.calls.at(index)?.[0]
  if (!request) throw new Error(`fetch call ${index} was not made`)
  return {
    url: request.url,
    method: request.method,
    body: (await request.clone().json()) as Record<string, unknown>,
  }
}

export function lastCall(): Promise<RecordedCall> {
  return callAt(-1)
}
