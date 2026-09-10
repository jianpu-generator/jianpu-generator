import { ChevronDownIcon, GitHubLogoIcon, Link2Icon, UpdateIcon } from '@radix-ui/react-icons'
import * as Toast from '@radix-ui/react-toast'
import { useCallback, useEffect, useRef, useState } from 'react'
import type { SyncedShareGithubAuthResult } from '../storage/syncedShareGithubAuth'
import { ResponsiveMenu } from './ResponsiveMenu'

interface SyncedShareButtonProps {
  isSynced: boolean
  syncedShareLink: string | null
  /** Whether the dedicated Synced Share GitHub sign-in connection is
   * present -- see `useSyncedShareOwner.ts`. Gates whether clicking "Sync"
   * starts a share directly or shows the sign-in prompt below (mockup
   * Screen 1). */
  isGithubConnected: boolean
  /** Cached GitHub username for the "Synced as @username" identity chip
   * (mockup Screen 3); `null` until connected. */
  githubLogin: string | null
  onStartSync: () => string | null
  onStopSync: () => void
  /** Opens the popup "sign in with GitHub" flow. Never itself starts a
   * share -- per §0, the user must click "Sync" again once connected. */
  onSignInWithGithub: () => Promise<SyncedShareGithubAuthResult>
  className?: string
}

type SignInPromptStatus =
  | { kind: 'idle' }
  | { kind: 'signing-in' }
  | { kind: 'done'; login: string }
  | { kind: 'failed'; error: string }

export function SyncedShareButton({
  isSynced,
  syncedShareLink,
  isGithubConnected,
  githubLogin,
  onStartSync,
  onStopSync,
  onSignInWithGithub,
  className = 'preview-export-btn',
}: SyncedShareButtonProps) {
  const [toastOpen, setToastOpen] = useState(false)
  const [menuOpen, setMenuOpen] = useState(false)
  // Controls the "Sync" popover shown when the user clicks "Sync" without
  // being GitHub-connected yet (mockup Screen 1). Kept separate from
  // `menuOpen` above, which is the *synced* state's own dropdown.
  const [signInPromptOpen, setSignInPromptOpen] = useState(false)
  const [signInStatus, setSignInStatus] = useState<SignInPromptStatus>({
    kind: 'idle',
  })
  const signInPromptRef = useRef<HTMLDivElement>(null)

  // Closes the sign-in prompt on an outside click/Escape, mirroring the
  // dismiss behavior Radix's `DropdownMenu` gives the "Synced" state's own
  // menu for free -- this popover is a plain positioned `div` (not a
  // `DropdownMenu`) since only `@radix-ui/react-popover`'s `Anchor`
  // supports positioning content next to an element that itself keeps a
  // plain, always-live `onClick` (a `DropdownMenu.Trigger`'s `onClick`
  // would be swallowed while the menu is closed), and that package isn't a
  // dependency of this project.
  useEffect(() => {
    if (!signInPromptOpen) return
    function handlePointerDown(event: PointerEvent) {
      if (!signInPromptRef.current?.contains(event.target as Node)) {
        setSignInPromptOpen(false)
      }
    }
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') setSignInPromptOpen(false)
    }
    document.addEventListener('pointerdown', handlePointerDown)
    document.addEventListener('keydown', handleKeyDown)
    return () => {
      document.removeEventListener('pointerdown', handlePointerDown)
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [signInPromptOpen])

  const copyUrl = useCallback(async (url: string) => {
    try {
      await navigator.clipboard.writeText(url)
      setToastOpen(true)
    } catch {
      window.prompt('Copy this link to share:', url)
    }
  }, [])

  const handleSyncClick = useCallback(() => {
    if (isGithubConnected) {
      const link = onStartSync()
      if (link) void copyUrl(link)
      return
    }
    setSignInStatus({ kind: 'idle' })
    setSignInPromptOpen(true)
  }, [isGithubConnected, onStartSync, copyUrl])

  const handleSignInWithGithub = useCallback(() => {
    setSignInStatus({ kind: 'signing-in' })
    void onSignInWithGithub().then((result) => {
      // Per §0's "no seamless OAuth-then-continue flow" decision: this never
      // starts a share on its own, regardless of outcome -- the user must
      // click "Sync" again once connected. A `'cancelled'`/`'blocked'`
      // outcome (popup closed/abandoned) just returns the prompt to idle,
      // matching "no dangling state".
      if (result.ok) {
        setSignInStatus({ kind: 'done', login: result.login })
      } else if (result.reason === 'error') {
        setSignInStatus({ kind: 'failed', error: result.error })
      } else {
        setSignInPromptOpen(false)
        setSignInStatus({ kind: 'idle' })
      }
    })
  }, [onSignInWithGithub])

  return (
    <div className="export-menu">
      {isSynced && githubLogin && (
        <span
          className="synced-share-identity-chip"
          data-testid="synced-share-identity-chip"
        >
          <GitHubLogoIcon aria-hidden="true" />
          Synced as @{githubLogin}
        </span>
      )}
      {isSynced ? (
        // Radix's DropdownMenuTrigger opens the menu on `pointerdown` and
        // `preventDefault()`s it whenever the menu is currently closed —
        // which, per spec, suppresses the `click` event that would
        // otherwise follow. So a Trigger can only ever toggle its own
        // menu; it can't also carry a "start sync" click handler for the
        // not-synced state below. Rendering the plain "Sync" button
        // outside any Trigger, and only wrapping this "Synced" button in
        // one once there's a menu for it to open, sidesteps that entirely.
        <ResponsiveMenu.Root open={menuOpen} onOpenChange={setMenuOpen}>
          <ResponsiveMenu.Trigger asChild>
            <button
              type="button"
              className={className}
              data-testid="synced-share-button"
              aria-label="Synced share options"
            >
              <UpdateIcon aria-hidden="true" />
              Synced
              <ChevronDownIcon
                className="export-menu-caret"
                aria-hidden="true"
              />
            </button>
          </ResponsiveMenu.Trigger>
          <ResponsiveMenu.Content
            className="export-menu-list"
            align="end"
            sideOffset={4}
            title="Synced share options"
          >
            <ResponsiveMenu.Item
              className="export-menu-item"
              data-testid="copy-synced-share-link-button"
              onSelect={() => {
                if (syncedShareLink) void copyUrl(syncedShareLink)
              }}
            >
              <Link2Icon aria-hidden="true" />
              Copy Synced Link
            </ResponsiveMenu.Item>
            <ResponsiveMenu.Item
              className="export-menu-item"
              data-testid="stop-sync-button"
              onSelect={onStopSync}
            >
              <svg
                width="15"
                height="15"
                viewBox="0 0 15 15"
                fill="currentColor"
                aria-hidden="true"
              >
                <rect x="2" y="2" width="11" height="11" rx="1.5" />
              </svg>
              Stop Sync
            </ResponsiveMenu.Item>
          </ResponsiveMenu.Content>
        </ResponsiveMenu.Root>
      ) : (
        // Not a `ResponsiveMenu`/`DropdownMenu.Trigger` (which opens on
        // `pointerdown`, unconditionally, per the comment on the "Synced"
        // branch above) -- the click must go straight to `handleSyncClick`
        // so the GitHub-connected case can start a share directly. The
        // sign-in prompt below is a plain positioned `div`, closed on an
        // outside click/Escape (see the effect above), rather than a
        // Radix primitive that supports anchoring next to an element with
        // its own live `onClick`.
        <div className="synced-share-signin-anchor" ref={signInPromptRef}>
          <button
            type="button"
            className={className}
            data-testid="synced-share-button"
            aria-label="Sync"
            title="Anyone with this link can view your latest saved score after reloading. Don't share it publicly."
            onClick={handleSyncClick}
          >
            <UpdateIcon aria-hidden="true" />
            Sync
          </button>
          {signInPromptOpen && (
            <div
              className="export-menu-list synced-share-signin-prompt"
              data-testid="synced-share-signin-prompt"
              role="dialog"
            >
              {signInStatus.kind === 'done' ? (
                <p>
                  Signed in as @{signInStatus.login}. Click "Sync" again to
                  start sharing.
                </p>
              ) : (
                <>
                  <p>Sign in with GitHub to start a Synced Share.</p>
                  {signInStatus.kind === 'failed' && (
                    <p
                      className="synced-share-signin-prompt-error"
                      data-testid="synced-share-signin-error"
                    >
                      {signInStatus.error}
                    </p>
                  )}
                  <button
                    type="button"
                    className="export-menu-item"
                    data-testid="synced-share-sign-in-with-github-button"
                    disabled={signInStatus.kind === 'signing-in'}
                    onClick={handleSignInWithGithub}
                  >
                    <GitHubLogoIcon aria-hidden="true" />
                    {signInStatus.kind === 'signing-in'
                      ? 'Signing in…'
                      : 'Sign in with GitHub'}
                  </button>
                </>
              )}
            </div>
          )}
        </div>
      )}
      <Toast.Provider swipeDirection="right" duration={3000}>
        <Toast.Root
          className="export-audio-toast"
          data-testid="synced-share-link-copied-toast"
          open={toastOpen}
          onOpenChange={setToastOpen}
        >
          <Toast.Description>Synced link copied</Toast.Description>
        </Toast.Root>
        <Toast.Viewport className="export-audio-toast-viewport" />
      </Toast.Provider>
    </div>
  )
}
