import type { BrowserContext, Page } from '@playwright/test'

// Cross-step state for `synced-share-idempotent-create.steps.ts`. The
// "original" link itself is read from `synced-share-button-state.ts`'s
// shared `syncedShareButtonState` (set by the existing "the synced link is
// copied" step, reused verbatim here) -- this file only holds what's new:
// the second ("separate browser context") owner session, which cloud-backed
// tab is currently active (so a later "a separate browser context loads the
// same cloud-backed file ..." step knows which file to open without
// threading it through every step's params), and each seeded file's real
// D1 id (see `seededFileIds` below).
export interface IdempotentCreateState {
  activeTabName: string | undefined
  secondContext: BrowserContext | undefined
  secondPage: Page | undefined
  secondSyncedShareLink: string | undefined
  /** Maps each `Given`-seeded cloud file's display name (extension
   * stripped) to its real D1 `files.id`, as returned by `seedCloudFile`.
   * Captured so the "different account" scenario can hand a *different*
   * signed-in account a synthetic `/files/list` entry carrying that exact
   * id -- `create_share` treats `external_file_id` as an opaque
   * client-supplied string (`handlers.rs::create_share` never checks it
   * against the `files` table), but `files.id` is a globally unique
   * primary key (`0003_files.sql`), so two different accounts can never
   * really own a row with the same id -- this is the only way to drive
   * that exact "same external_file_id, different owner" request shape
   * through the real worker instead of an impossible D1 state. See
   * `loadSecondContextOnCloudFile`'s own doc comment. */
  seededFileIds: Record<string, string>
}

export const idempotentCreateState: IdempotentCreateState = {
  activeTabName: undefined,
  secondContext: undefined,
  secondPage: undefined,
  secondSyncedShareLink: undefined,
  seededFileIds: {},
}

export function resetIdempotentCreateState(): void {
  idempotentCreateState.activeTabName = undefined
  idempotentCreateState.secondContext = undefined
  idempotentCreateState.secondPage = undefined
  idempotentCreateState.secondSyncedShareLink = undefined
  idempotentCreateState.seededFileIds = {}
}

/** Deterministic per-name content for a cloud file seeded by "a cloud file
 * named {string} is seeded for idempotent sharing" -- distinct titles just
 * make a failing assertion's diff readable, the content itself isn't
 * asserted on by any scenario in this feature. */
export function idempotentSharingSource(name: string): string {
  const title = name.replace(/\.jianpu$/, '')
  return [
    '# metadata',
    `title = "${title}"`,
    '',
    '# parts',
    'Melody = notes',
    '',
    '# score',
    '(time=4/4 key=C4 bpm=120)',
    '1 2 3 4',
  ].join('\n')
}
