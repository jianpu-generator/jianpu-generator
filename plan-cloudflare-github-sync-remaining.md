# Cloudflare GitHub Sync — Remaining Work

Implementation landed on branch `feature/cloudflare-github-sync`. This file tracks what is still required before the feature is fully live.

## One-time infrastructure setup

Do these in Cloudflare and GitHub dashboards (not in code):

1. **Cloudflare account** — free tier is enough; create a Pages project named `jianpu-generator`.
2. **GitHub OAuth App** — public client, `repo` scope (private repos). Register callback URLs:
   - `https://jianpu-generator.pages.dev/api/github/callback`
   - `http://localhost:8788/api/github/callback` (local `wrangler pages dev`)
3. **GitHub repo secrets** (Settings → Secrets → Actions):
   - `CLOUDFLARE_API_TOKEN`
   - `CLOUDFLARE_ACCOUNT_ID`
4. **Cloudflare Pages secrets** (project → Settings → Environment variables):
   - `GITHUB_CLIENT_ID`
   - `GITHUB_CLIENT_SECRET`
   - `SESSION_SECRET`

Until secrets are set, the Cloudflare deploy workflow will fail on deploy (build should still pass).

## Deploy

- Merge `feature/cloudflare-github-sync` into `master` — both `.github/workflows/pages.yml` and `cloudflare-pages.yml` deploy on push to `master`.
- After merge, confirm:
  - GitHub Pages build has `VITE_ENABLE_GITHUB_SYNC=false` (Local workspace + banner only).
  - Cloudflare Pages build has `VITE_ENABLE_GITHUB_SYNC=true` (full Local + GitHub workspaces).

## Manual verification

After deploy to Cloudflare URL:

1. Open the Cloudflare URL (not GitHub Pages) — GitHub sync is impossible on `*.github.io`.
2. Switch to **GitHub** workspace → **Connect with GitHub** → authorize.
3. Create or edit a score tab → wait for autosave indicator → confirm file appears in private `jianpu-scores` repo on GitHub.
4. Optional local dev: `cd web && pnpm build && npx wrangler pages dev dist` with secrets in `.dev.vars`.

## Small UX gaps (optional polish)

- **OAuth error UI** — callback redirects to `/?github_error=<code>` on failure; App does not surface this yet.
- **Disconnect button** — `POST /api/github/logout` exists; no UI to clear session.

## Deferred / out of scope (v1)

- Playwright E2E for GitHub sync (`web/e2e/github-sync.spec.ts`) — skipped for now; tests were timing out and need mock/routing fixes before re-enabling.
- Local → GitHub file migration (“Copy to GitHub”).
- Conflict UI (last-write-wins only).
- Sync on GitHub Pages origin.

## Reference

- Overview: `plan-cloudflare-github-sync.md`
- Step plans: `plan-cloudflare-github-sync-1-ci-and-flags.md` … `plan-cloudflare-github-sync-6-autosave-and-e2e.md`
