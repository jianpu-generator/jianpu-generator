// Relays POST /login/device/code to start the GitHub OAuth device flow.
//
// This endpoint only needs the (non-secret) client ID, but GitHub still
// doesn't send CORS headers for it, so it's proxied here purely to work
// around that. No client secret is involved in this call.

import { Env, handlePreflight, corsHeadersFor, rejectDisallowedOrigin } from "../lib/cors";

const GITHUB_DEVICE_CODE_URL = "https://github.com/login/device/code";

export const onRequestOptions: PagesFunction<Env> = async (context) => {
  return handlePreflight(context.request, context.env) ?? new Response(null, { status: 204 });
};

export const onRequestPost: PagesFunction<Env> = async (context) => {
  const { request, env } = context;

  const rejection = rejectDisallowedOrigin(request, env);
  if (rejection) {
    return rejection;
  }

  let scope: string | undefined;
  try {
    const body = (await request.json()) as { scope?: string };
    scope = body?.scope;
  } catch {
    // No/invalid JSON body is fine; scope is optional.
  }

  const upstreamResponse = await fetch(GITHUB_DEVICE_CODE_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify({
      client_id: env.GITHUB_CLIENT_ID,
      ...(scope ? { scope } : {}),
    }),
  });

  const headers = corsHeadersFor(request.headers.get("Origin"), env) ?? new Headers();
  headers.set("Content-Type", "application/json");

  return new Response(await upstreamResponse.text(), {
    status: upstreamResponse.status,
    headers,
  });
};
