import { beforeEach, vi } from 'vitest'

/** Shared `fetch` mock + response/body helpers for `cloudBackend.test.ts`
 * and `cloudBackendRetry.test.ts` -- split out so each test file stays
 * under this repo's 400-line cap while keeping identical setup. Vitest
 * isolates each test file's module graph by default, so the `fetchMock`
 * instance below is fresh per test file despite the shared import. */
export const fetchMock = vi.fn()

export function jsonResponse(status: number, body: unknown): Response {
  return {
    status,
    json: () => Promise.resolve(body),
  } as unknown as Response
}

export function emptyResponse(status: number): Response {
  return {
    status,
    json: () => Promise.reject(new Error('no body')),
  } as unknown as Response
}

beforeEach(() => {
  fetchMock.mockReset()
  vi.stubGlobal('fetch', fetchMock)
})

export const config = { token: 'test-token', workerHost: 'localhost:8787' }

export function lastCall(): { url: string; init: RequestInit } {
  const call = fetchMock.mock.calls.at(-1)
  if (!call) throw new Error('fetch was not called')
  return { url: call[0] as string, init: call[1] as RequestInit }
}

export function parsedBody(init: RequestInit): Record<string, unknown> {
  return JSON.parse(init.body as string) as Record<string, unknown>
}
