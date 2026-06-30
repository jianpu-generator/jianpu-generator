# Step 5: Dual workspace UI and stores

## Goal
Split file store into Local vs GitHub backends; workspace switcher; Connect prompt when GitHub workspace selected but not connected. Gate all GitHub UI with `enableGitHubSync` from `web/src/env.ts`.

## Changes

### Hooks
- Rename `useFileStore` → `useLocalFileStore` (keep re-export alias `useFileStore` for minimal churn OR update App only)
- New `useGitHubFileStore()`: loads from `GET /api/github/store` when connected; same `FileStoreState` + setter API as local
- New `useGitHubSession()`: polls/fetches `/api/github/session`

### Components (Radix where interactive)
- `WorkspaceSwitcher`: Local | GitHub tabs (only if `enableGitHubSync`)
- `GitHubConnectButton`: navigates to `/api/github/login`
- When GitHub workspace + disconnected: show connect prompt instead of FileList

### `web/src/App.tsx`
- Workspace state: `'local' | 'github'`
- Wire appropriate store hook based on workspace
- Part toggles: keep local cache key; optionally prefix storage key per workspace later (v1: separate in-memory only for GitHub is OK if same key — plan says local-only per workspace; use `jianpu:part-toggles:local` vs `jianpu:part-toggles:github` or pass workspace to partToggleCache)

### Read-only GitHub edits OK for this step
Autosave comes in step 6 — but store mutations should work in memory; optional manual refresh from repo.

## Verification
```sh
cd web && pnpm build && pnpm test
```
With mocked fetch in dev, switcher appears when `VITE_ENABLE_GITHUB_SYNC=true`.

## Out of scope
- Debounced autosave (step 6)
- Playwright (step 6)
