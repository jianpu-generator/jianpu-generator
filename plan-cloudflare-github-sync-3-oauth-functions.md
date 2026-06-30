# Step 3: OAuth Pages Functions

## Goal
Implement session-based GitHub OAuth under `web/functions/api/github/`. Reference: customkeymap functions pattern (encrypted httpOnly cookie, PKCE + state).

## Routes

| Route | Method | Purpose |
|-------|--------|---------|
| `/api/github/login` | GET | Redirect to GitHub OAuth (PKCE + state cookie) |
| `/api/github/callback` | GET | Exchange code, set session cookie, redirect to `/` |
| `/api/github/logout` | POST | Clear session |
| `/api/github/session` | GET | JSON `{ connected, username?, repo? }` |

## Shared utilities (`web/functions/_lib/` or similar)
- Session encrypt/decrypt using `SESSION_SECRET` env
- Cookie names, Secure + HttpOnly + SameSite=Lax
- Read env: `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`, `SESSION_SECRET`

## Callback behavior
- OAuth scope: `repo` (private repos)
- On first connect: attempt to create private repo `jianpu-scores`; if exists (409), reuse
- Store in session: `accessToken`, `username`, `repo: 'jianpu-scores'`
- Do NOT implement file CRUD yet (step 4)

## Types
Use `@cloudflare/workers-types` if needed; keep functions in TypeScript under `web/functions/`.

## Verification
- Typecheck/build functions if wrangler supports it
- Document local dev: `pnpm build && npx wrangler pages dev dist` with env vars

## Out of scope
- store/files/manifest routes (step 4)
- Frontend Connect button (step 5)
