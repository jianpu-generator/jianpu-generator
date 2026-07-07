# TODO: remaining e2e coverage gaps

Ranked by real user-facing risk (a bug here would actually reach a user
undetected), not by ease of writing the test. See `web/e2e/` for existing
specs and `web/e2e/github-contents-mock.ts` for the GitHub Contents API mock
helper used by the `*-github.spec.ts` files.

- [x] **GitHub 409 conflict resolution** (`StorageSettingsModal.tsx:89`
      `resolveGithubConflict`, `handleResolveConflict` at line 277) — the
      highest-risk gap: a regression here could silently destroy a user's
      unsaved edits. Extend `mockGithubContentsApi` (or add a variant) so a
      `PUT` returns `409`, driving `githubBackend.ts:292`'s
      `{ kind: 'conflict', path }`. Then assert:
      - `data-testid="conflict-banner"` appears with the conflicting path.
      - Clicking "Overwrite mine" (`onClick={() =>
        handleResolveConflict('overwrite-mine')}`) re-PUTs the in-memory
        content and the banner clears.
      - Clicking "Discard mine" reloads the backend's listing and the
        editor's content is replaced with what the mock now serves for that
        path (assert the visible source text changed, not just that the
        banner cleared).

- [x] **GitHub OAuth connect/disconnect device flow**
      (`StorageSettingsModal.tsx:228` `handleConnect`, `:254`
      `handleDisconnect`). Every existing `*-github.spec.ts` test seeds
      `jianpu:github-auth:v1` via `page.addInitScript`, so `handleConnect`'s
      device-code flow (`connectWithDeviceFlow`, `GITHUB_OAUTH_PROXY_URL`)
      has never run through the UI. Mock the device-flow proxy + GitHub's
      `users.getAuthenticated`/`repos.get` endpoints via `page.route` and
      assert:
      - Clicking the "GitHub repository" radio with no stored auth shows
        `data-testid="github-connect"`, then clicking its button shows
        `data-testid="device-verification"` with the `user_code`.
      - Once the mocked proxy resolves, `data-testid="github-connected"`
        appears with the right `@username`.
      - Clicking "Disconnect" clears `jianpu:github-auth:v1` and reverts to
        the local backend (assert via `switchBackend`'s visible effect, e.g.
        the radio selection and that a subsequent edit doesn't hit
        `API_PREFIX`).

- [x] **PDF export** (`Preview.tsx:359` `canExportPdf`/`onExportPdf`,
      `useJianpuWorker.ts:490` `exportPdf`, `:526` `exportSplitPdf`,
      `workerHelpers.ts:41` `downloadPdf`). Zero e2e coverage on a primary
      output button. Using Playwright's download event
      (`page.waitForEvent('download')`), assert:
      - Clicking "Export PDF" with a valid document produces a downloaded
        file with a non-trivial size (catches a silently-empty/corrupt PDF).
      - Clicking "Export parts" (`canExportSplitPdf`, requires
        `partsCount > 0`) on a multi-part score produces a download too —
        this path is separate from `exportPdf` and could regress
        independently.
      - Both buttons are disabled while `rendering`/`exporting` is true
        (`disabled={!canExportPdf}` / `!canExportSplitPdf`).

- [ ] **Cmd/Ctrl+S force-save shortcut** (`App.tsx:212`
      `modifier && event.key.toLowerCase() === 's'` → `forceSaveRef.current()`
      / `forceSave`). Shares the same `onKeyDown` handler as the already-
      tested play shortcut (`web/e2e/cmd-enter-play.spec.ts`), but only the
      play branch is covered — a regression in the modifier/key check could
      break saving without any test noticing. Assert that pressing
      Meta/Ctrl+S triggers a save (e.g. via the GitHub mock, assert a `PUT`
      fires immediately rather than waiting for autosave's debounce).

- [ ] **Playback actually plays** (`Preview.tsx`'s `playMeasureRef` /
      `playSelectedMeasures` in `useJianpuWorker.ts`). Existing
      `cmd-enter-play.spec.ts` only asserts the no-op case (no measure
      selected). Add a case with a valid `selectedMeasureRange` and
      `soundfontReady`, and assert the "playing" state actually engages
      (e.g. the play/stop control's visible state flips, or
      `measureAudioPlaying` becomes true) rather than only checking the
      shortcut didn't crash.

- [ ] **Undo/redo across file switches with unsaved edits** (`fileStore.ts`
      — only unit-tested today; `restore-file-github.spec.ts` etc. cover
      single-file backend round-trips but not this). Assert that editing
      file A, switching to file B without saving, switching back to A,
      still shows A's edit (or the correct discard/keep behavior per
      `fileStore.ts`'s actual contract) — this is where autosave-per-active-
      file bugs like the one fixed for shared-import (see
      `TODO-e2e-github-storage.md`'s import entry) tend to hide.

## Not worth adding right now

- Individual GitHub error-path tests (500s, rate-limit banner) per
  operation (create/rename/delete/duplicate) — real gap, but lower priority
  than the conflict-resolution test above since it's the same banner logic
  (`errorBannerMessage`) already partially exercised; add only if a bug
  surfaces there.
- `SoundfontSearchModal` — real UI gap (zero e2e references) but low risk:
  it's a search/filter list, not a data-mutating flow.

## Cleanup

- [ ] Remove `web/e2e/diag.spec.ts` — it's a console-log debugging scratch
      file ("what does localStorage contain when share URL loads?"), not an
      assertion-driven test, and it overlaps `share.spec.ts` which already
      covers the real behavior.
