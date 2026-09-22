/** Thrown by `request()` when `fetch` itself rejects (offline, DNS failure,
 * aborted request) -- distinct from a successful fetch that resolved to a
 * non-2xx status (`HttpStatusError` below). */
export class NetworkFailure extends Error {}

/** Thrown by `request()` for any non-2xx response, carrying the parsed JSON
 * body (if any) so callers can inspect it for a specific shape (a
 * revision-conflict body, a `{code: "name_taken"}` body, ...) without a
 * second fetch. */
export class HttpStatusError extends Error {
  readonly httpStatus: number
  readonly body: unknown

  constructor(httpStatus: number, body: unknown) {
    super(`cloud storage request failed with status ${httpStatus}`)
    this.httpStatus = httpStatus
    this.body = body
  }
}

export function isConflictBody(
  body: unknown,
): body is { currentRevision: number } {
  return (
    typeof body === 'object' &&
    body !== null &&
    typeof (body as { currentRevision?: unknown }).currentRevision === 'number'
  )
}

function isNameTakenBody(body: unknown): body is { code: 'name_taken' } {
  return (
    typeof body === 'object' &&
    body !== null &&
    (body as { code?: unknown }).code === 'name_taken'
  )
}

export function isNameTakenError(error: unknown): boolean {
  return (
    error instanceof HttpStatusError &&
    error.httpStatus === 409 &&
    isNameTakenBody(error.body)
  )
}

/** POSTs a JSON body to a `/files/*` route and returns the parsed JSON
 * response (or `null` for a `204 No Content`). Throws `NetworkFailure` on a
 * `fetch` rejection, `HttpStatusError` on any non-2xx response -- every
 * caller classifies one of those two, never a raw `fetch` result. */
export async function request(
  origin: string,
  path: string,
  body: object,
): Promise<unknown> {
  let response: Response
  try {
    response = await fetch(`${origin}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    })
  } catch {
    throw new NetworkFailure('fetch failed')
  }
  if (response.status === 204) return null
  const json: unknown = await response.json().catch(() => null)
  if (response.status >= 200 && response.status < 300) return json
  throw new HttpStatusError(response.status, json)
}
