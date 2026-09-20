import type { BrowserContext, Page } from '@playwright/test'

// Cross-step state for `synced-share-idempotent-create.steps.ts`. The
// "original" link itself is read from `synced-share-button-state.ts`'s
// shared `syncedShareButtonState` (set by the existing "the synced link is
// copied" step, reused verbatim here) -- this file only holds what's new:
// the second ("separate browser context") owner session, and which
// GitHub-backed tab is currently active (so a later "a separate browser
// context loads the same GitHub-backed file ..." step knows which file to
// open without threading it through every step's params).
export interface IdempotentCreateState {
  activeTabName: string | undefined
  secondContext: BrowserContext | undefined
  secondPage: Page | undefined
  secondSyncedShareLink: string | undefined
}

export const idempotentCreateState: IdempotentCreateState = {
  activeTabName: undefined,
  secondContext: undefined,
  secondPage: undefined,
  secondSyncedShareLink: undefined,
}

export function resetIdempotentCreateState(): void {
  idempotentCreateState.activeTabName = undefined
  idempotentCreateState.secondContext = undefined
  idempotentCreateState.secondPage = undefined
  idempotentCreateState.secondSyncedShareLink = undefined
}

/** Deterministic per-path content for a GitHub-backed file seeded by "the
 * GitHub repo is seeded with a file named {string} for idempotent sharing"
 * -- distinct titles just make a failing assertion's diff readable, the
 * content itself isn't asserted on by any scenario in this feature. */
export function idempotentSharingSource(path: string): string {
  const title = path.replace(/^scores\//, '').replace(/\.jianpu$/, '')
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
