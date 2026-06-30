# Step 6: GitHub autosave and E2E tests

## Goal
Debounced push to GitHub on store changes; sync status UI; Playwright tests with mocked `/api/github/*`.

## Changes

### `web/src/hooks/useGitHubAutosave.ts`
- Debounce ~1500ms after store changes (similar to `useJianpuWorker.ts`)
- Track dirty file paths; PUT only changed files; PATCH manifest when metadata changes
- Status: `idle | saving | saved | error`
- On switch to GitHub workspace: full pull from `/api/github/store`

### `web/src/components/SyncStatusIndicator.tsx`
Show status in GitHub workspace header.

### Playwright (`web/e2e/` or existing test dir)
Mock routes:
- `/api/github/session` — connected/disconnected scenarios
- `/api/github/store`, files PUT — autosave debounce
- Workspace isolation: Local tabs unchanged when switching workspaces

## Verification
```sh
cd web && pnpm test && pnpm exec playwright test
```

## Out of scope
- Local → GitHub migration
- Conflict UI
