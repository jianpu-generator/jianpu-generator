# GitHub Backend Integration — Review TODOs

Findings from code review of the `github-backend-integrated` branch (vs `master`).

## Correctness

- [x] **Race condition clobbers concurrent edits** — `web/src/App.tsx:258`
  Fixed. `handleCreate`/`handleDuplicate`/`handleRename`/`handleDelete`/`handleRestore` now
  capture the pre-await snapshot as `base`, await `backend.xxxFile(base)` to get `next`, and
  commit via `setStore(prev => mergeBackendResult(prev, base, next))`. The new
  `mergeBackendResult` (`web/src/fileStore.ts`) diffs `base` against `next` to find only the
  keys the operation actually added/removed/renamed, and applies just that diff onto the
  latest `prev` — so a concurrent edit or a GitHub `load()` resolving mid-await is preserved
  instead of clobbered. Covered by `web/src/fileStore.test.ts`.

- [ ] **Unhandled promise rejections on GitHub errors** — `web/src/App.tsx:257`
  None of the file-operation handlers (nor `StorageSettingsModal`'s auth-status check)
  have a `try`/`catch`. Any thrown GitHub error (rate-limit, offline, etc.) becomes an
  unhandled rejection — the action silently fails with no user-facing error.

- [ ] **Silent 422 failure on rename/restore name collisions** — `web/src/storage/githubBackend.ts:355`
  `renameFile`/`restoreFile` create the destination with no `sha`, relying only on local
  in-memory uniqueness. A cross-tab/device name collision returns HTTP 422, which
  `classifyError` doesn't recognize — it's bucketed as `'unknown'` and never surfaced
  in the UI.

- [x] **fileId regenerated on every load(), resetting part-toggle settings** — `web/src/storage/githubBackend.ts:317`
  Fixed. Name -> file-ID mappings are now persisted to `localStorage`, keyed per
  `owner/repo` (`readStoredFileIds`/`writeStoredFileIds`). `load()` reuses the stored ID
  for any name it has seen before instead of minting a fresh UUID, and every structural
  op (`createFile`/`duplicateFile`/`renameFile`/`restoreFile`) persists its updated
  mapping immediately so IDs survive a reload even before the next `load()`. Covered by
  new tests in `web/src/storage/githubBackend.test.ts`.

- [ ] **403 always misclassified as rate-limited** — `web/src/storage/githubBackend.ts:248`
  Every HTTP 403 is classified as `'rate-limited'`, even though GitHub also returns 403
  for insufficient scope or org-restricted access. Affected users get a permanently
  misleading "rate limit" banner with no reconnect path.

- [ ] **Unguarded env var crash in CORS check** — `cf-oauth-proxy/functions/lib/cors.ts:15`
  `env.ALLOWED_ORIGINS.split(",")` has no guard and `wrangler.toml` defines no default.
  If the env var is ever unset, the origin-check security mechanism itself throws a 500
  instead of failing closed with a clean 403.

## Cleanup

- [ ] **Duplicated `statusOf` helper** — `web/src/storage/githubBackend.ts:69`
  Defined byte-for-byte identically in `StorageSettingsModal.tsx:44`. Export once and
  share instead of maintaining two copies.

## Noted but not actioned

- `sha` is refetched via a separate network call before every save/delete in
  `githubBackend.ts` (`putFile`, line ~216), doubling API calls per write. Left as-is —
  the code comments document this as an intentional "v1" tradeoff to avoid stale-cache
  bugs across rename/restore/delete cycles, not an oversight.
