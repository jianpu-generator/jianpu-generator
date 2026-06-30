export function jsonResponse(body: unknown, init: ResponseInit = {}): Response {
  const headers = new Headers(init.headers)
  if (!headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json')
  }
  return new Response(JSON.stringify(body), { ...init, headers })
}

export function redirectResponse(
  location: string,
  headers: HeadersInit = {},
): Response {
  return new Response(null, {
    status: 302,
    headers: {
      Location: location,
      ...headers,
    },
  })
}

export function errorRedirect(requestUrl: URL, error: string): Response {
  const location = new URL('/', requestUrl.origin)
  location.searchParams.set('github_error', error)
  return redirectResponse(location.toString())
}
