# Step 2: Manifest ↔ FileStoreState mapping

## Goal
Pure TypeScript module mapping between `FileStoreState` (see `web/src/fileStore.ts`) and the GitHub repo layout from the parent plan. Unit tests only — no API calls.

## Repo layout
```
jianpu-scores/
  .jianpu/manifest.json
  scores/*.jianpu
  bin/*.jianpu
```

## Changes

### `web/src/github/manifest.ts` (new)
Types:
- `GitHubManifest`: `{ active, fileIds, bin: string[] }` (bin is names only; content lives under `bin/`)

Functions:
- `fileStoreToManifest(state: FileStoreState): GitHubManifest`
- `manifestAndFilesToFileStore(manifest, scoreFiles: Record<string, string>, binFiles: Record<string, string>): FileStoreState`
- `scorePath(name: string): string` → `scores/${name}`
- `binPath(name: string): string` → `bin/${name}`
- Exclude demo file (`DEMO_FILE_NAME`) from sync — manifest only tracks user files

### `web/src/github/manifest.test.ts` (new)
Vitest tests: round-trip, rename preserves fileIds, bin names, active file.

## Verification
```sh
cd web && pnpm test -- manifest
```

## Out of scope
- Pages Functions (step 4)
- Hooks (step 5)
