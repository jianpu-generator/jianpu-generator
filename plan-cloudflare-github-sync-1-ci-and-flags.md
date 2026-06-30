# Step 1: CI, wrangler, and GitHub sync feature flag

## Goal
Wire dual deploy (GitHub Pages + Cloudflare Pages) and `VITE_ENABLE_GITHUB_SYNC` / `VITE_BASE_PATH` build flags. Add a small banner on GitHub Pages pointing users to Cloudflare for sync. No OAuth or file-store changes yet.

## Changes

### `.github/workflows/pages.yml`
Add to build step env:
```yaml
VITE_ENABLE_GITHUB_SYNC: 'false'
```
(Keep existing `VITE_BASE_PATH: /jianpu-generator/`)

### `.github/workflows/cloudflare-pages.yml` (new)
- Trigger: push to master + workflow_dispatch
- Same build as pages.yml (pnpm, rust wasm, pnpm build in web/)
- Build env: `VITE_BASE_PATH: /`, `VITE_ENABLE_GITHUB_SYNC: 'true'`
- Deploy with `cloudflare/wrangler-action@v3`, working-directory `web/`
- Secrets: `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID`

### `web/wrangler.toml` (new)
```toml
name = "jianpu-generator"
pages_build_output_dir = "dist"
compatibility_date = "2024-01-01"
```

### `web/src/env.ts` (new)
Export typed helpers:
- `enableGitHubSync`: `import.meta.env.VITE_ENABLE_GITHUB_SYNC === 'true'`
- `cloudflareAppUrl`: constant string for the banner link (use `https://jianpu-generator.pages.dev` or env `VITE_CLOUDFLARE_APP_URL` with that default)

### `web/src/vite-env.d.ts`
Add `VITE_ENABLE_GITHUB_SYNC` and optional `VITE_CLOUDFLARE_APP_URL` to ImportMetaEnv.

### `web/src/App.tsx`
When `!enableGitHubSync`, show a dismissible or static subtle banner: GitHub sync requires the Cloudflare deployment; link to `cloudflareAppUrl`.

## Verification
```sh
cd web && pnpm build
VITE_ENABLE_GITHUB_SYNC=false VITE_BASE_PATH=/jianpu-generator/ pnpm build
VITE_ENABLE_GITHUB_SYNC=true VITE_BASE_PATH=/ pnpm build
```
Both builds succeed. Banner visible only when sync disabled.

## Out of scope
- `web/functions/` (step 3+)
- Workspace switcher (step 5)
