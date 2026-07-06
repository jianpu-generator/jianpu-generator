# jianpu-oauth-proxy

**This is a separate Cloudflare Pages project. It is NOT part of the `web/`
build** — it does not get bundled, built, or deployed by anything under
`web/`, and `web/`'s Vite/pnpm pipeline does not know this directory exists.
It is deployed as its own, independent Cloudflare Pages project.

## Why this exists

GitHub's OAuth **device flow** endpoints —

- `POST https://github.com/login/device/code`
- `POST https://github.com/login/oauth/access_token`

— do not send CORS headers, so a static site (like the GitHub Pages-hosted
`web/` app) cannot call them directly from the browser. Every other GitHub
API call the app needs (Contents API, Trees API, `/user`, `/user/repos`)
supports CORS directly with a bearer token and is called straight from the
browser — only these two calls are proxied here.

There is deliberately no revoke/disconnect endpoint in this proxy;
disconnecting is handled locally (client-side) in a later step.

## Layout

```
functions/
  device/code.ts     relays POST /login/device/code
  oauth/token.ts      relays POST /login/oauth/access_token (device-code and refresh-token grants)
  lib/cors.ts         shared origin-check + CORS header helper used by both routes above
wrangler.toml         minimal Cloudflare Pages Functions config
public/               empty; Pages requires an output directory even for a Functions-only project
```

## Environment variables

Configure these in the Cloudflare dashboard for this Pages project
(Settings → Environment variables), not in this repo:

| Name                    | Secret? | Purpose                                                             |
| ------------------------ | ------- | -------------------------------------------------------------------- |
| `GITHUB_CLIENT_ID`       | no      | The OAuth App's client ID.                                          |
| `GITHUB_CLIENT_SECRET`   | yes     | The OAuth App's client secret. Never sent to the browser, never logged. |
| `ALLOWED_ORIGINS`        | no      | Comma-separated list of allowed origins (e.g. `http://localhost:5173,https://<user>.github.io`). Requests from any other `Origin` are rejected with 403. |

## Deploying

`.github/workflows/oauth-proxy.yml` deploys this project to Cloudflare Pages
automatically on every push to `master` that touches `cf-oauth-proxy/`, using
[`cloudflare/wrangler-action`](https://github.com/cloudflare/wrangler-action).
It requires two repo secrets (Settings → Secrets and variables → Actions):

| Name                     | Where to get it                                                        |
| ------------------------ | ----------------------------------------------------------------------- |
| `CLOUDFLARE_API_TOKEN`   | Cloudflare dashboard → My Profile → API Tokens → Create Token, with "Cloudflare Pages — Edit" permission for this account. |
| `CLOUDFLARE_ACCOUNT_ID`  | Cloudflare dashboard → right sidebar of any domain/Workers & Pages overview page. |

The `GITHUB_CLIENT_ID` / `GITHUB_CLIENT_SECRET` / `ALLOWED_ORIGINS`
environment variables above are Cloudflare Pages project settings, not
GitHub Actions secrets — they're unaffected by CI and only need to be set
once (or updated) via the dashboard or `wrangler pages secret put`, per the
note under the table above.

Alternatively, this project is Functions-only (see the empty `public/`
directory), so it can also be deployed by connecting this directory as its
own Cloudflare Pages project in the dashboard, without using the workflow:

1. In the Cloudflare dashboard, create a new Pages project pointing at this
   repo, with **`cf-oauth-proxy`** set as the project's root/build directory
   and `public` as the build output directory (no build command needed).
2. Set the three environment variables above under that project's settings.
3. Deploy.

Alternatively, deploy directly from the CLI with [Wrangler](https://developers.cloudflare.com/workers/wrangler/):

```sh
cd cf-oauth-proxy
npx wrangler pages deploy public --project-name jianpu-oauth-proxy
```

(Environment variables/secrets still need to be set once via the dashboard,
or with `npx wrangler pages secret put GITHUB_CLIENT_SECRET`.)

## Security notes

- The client secret is only ever read from `context.env` inside these
  Functions and is only ever sent to `github.com` — never to the browser,
  never logged.
- Token response bodies are relayed to the caller but are never logged or
  otherwise inspected server-side.
- Every request is checked against `ALLOWED_ORIGINS` before anything is
  relayed to GitHub; mismatched or missing `Origin` headers get a 403.
