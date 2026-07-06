// Shared origin-check + CORS header helper for the OAuth device-flow proxy.
//
// GitHub's OAuth device-flow endpoints do not send CORS headers, so this
// proxy adds them itself, but only for the single origin configured via the
// ALLOWED_ORIGIN environment variable. Requests from any other Origin are
// rejected before we relay anything to GitHub.

export interface Env {
  GITHUB_CLIENT_ID: string;
  GITHUB_CLIENT_SECRET: string;
  ALLOWED_ORIGIN: string;
}

/**
 * Returns the CORS headers to attach to a response when `origin` matches the
 * configured ALLOWED_ORIGIN, or `null` if the origin is not allowed.
 */
export function corsHeadersFor(origin: string | null, env: Env): Headers | null {
  if (!origin || origin !== env.ALLOWED_ORIGIN) {
    return null;
  }

  const headers = new Headers();
  headers.set("Access-Control-Allow-Origin", origin);
  headers.set("Vary", "Origin");
  headers.set("Access-Control-Allow-Methods", "POST, OPTIONS");
  headers.set("Access-Control-Allow-Headers", "Content-Type, Accept");
  return headers;
}

/**
 * Rejects the request with a 403 if its Origin header does not match
 * ALLOWED_ORIGIN. Returns null when the request is allowed to proceed.
 */
export function rejectDisallowedOrigin(request: Request, env: Env): Response | null {
  const origin = request.headers.get("Origin");
  const headers = corsHeadersFor(origin, env);
  if (!headers) {
    return new Response("Forbidden origin", { status: 403 });
  }
  return null;
}

/**
 * Handles a CORS preflight (OPTIONS) request. Returns a Response if this
 * request was a preflight (whether allowed or rejected), or null if the
 * caller should continue processing a non-OPTIONS request.
 */
export function handlePreflight(request: Request, env: Env): Response | null {
  if (request.method !== "OPTIONS") {
    return null;
  }
  const origin = request.headers.get("Origin");
  const headers = corsHeadersFor(origin, env);
  if (!headers) {
    return new Response("Forbidden origin", { status: 403 });
  }
  return new Response(null, { status: 204, headers });
}
