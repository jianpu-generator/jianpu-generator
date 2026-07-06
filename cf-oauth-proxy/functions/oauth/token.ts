// Relays POST /login/oauth/access_token to exchange a device code (or
// refresh token) for an access token.
//
// Handles both grant types the frontend needs:
//   - grant_type=urn:ietf:params:oauth:grant-type:device_code (device flow polling)
//   - grant_type=refresh_token (refreshing an expired access token)
//
// The client secret is injected here, server-side, and is never sent to or
// read from the browser. The token response body is relayed as-is to the
// caller but is never logged.

import { Env, handlePreflight, corsHeadersFor, rejectDisallowedOrigin } from "../lib/cors";

const GITHUB_TOKEN_URL = "https://github.com/login/oauth/access_token";

interface TokenRequestBody {
  grant_type?: string;
  device_code?: string;
  refresh_token?: string;
}

export const onRequestOptions: PagesFunction<Env> = async (context) => {
  return handlePreflight(context.request, context.env) ?? new Response(null, { status: 204 });
};

export const onRequestPost: PagesFunction<Env> = async (context) => {
  const { request, env } = context;

  const rejection = rejectDisallowedOrigin(request, env);
  if (rejection) {
    return rejection;
  }

  let body: TokenRequestBody;
  try {
    body = (await request.json()) as TokenRequestBody;
  } catch {
    return new Response("Invalid JSON body", { status: 400 });
  }

  if (body.grant_type !== "urn:ietf:params:oauth:grant-type:device_code" && body.grant_type !== "refresh_token") {
    return new Response("Unsupported grant_type", { status: 400 });
  }

  const upstreamResponse = await fetch(GITHUB_TOKEN_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify({
      client_id: env.GITHUB_CLIENT_ID,
      client_secret: env.GITHUB_CLIENT_SECRET,
      grant_type: body.grant_type,
      ...(body.device_code ? { device_code: body.device_code } : {}),
      ...(body.refresh_token ? { refresh_token: body.refresh_token } : {}),
    }),
  });

  const headers = corsHeadersFor(request.headers.get("Origin"), env) ?? new Headers();
  headers.set("Content-Type", "application/json");

  // Relay the response body as-is; deliberately not logged or inspected here.
  return new Response(await upstreamResponse.text(), {
    status: upstreamResponse.status,
    headers,
  });
};
