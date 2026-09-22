import {
  ExitIcon,
  ExternalLinkIcon,
  GitHubLogoIcon,
} from '@radix-ui/react-icons'
import * as Popover from '@radix-ui/react-popover'
import { useCallback, useState } from 'react'
import type { SyncedShareGithubAuthResult } from '../storage/accountAuthPopup'
import './ProfileControl.css'

/**
 * The header's persistent account control (see `HANDOFF-account-chip.md`)
 * -- the only glanceable indicator anywhere in the app that the user is
 * signed in with GitHub, and the only place that signs out. `ShareModal`'s
 * Synced-link tab and `StorageSettingsModal` still offer "Sign in with
 * GitHub" as a shortcut (same popup flow as this component's own button),
 * but neither renders a sign-out control any more -- both just show a
 * plain status line once connected.
 */
export interface ProfileControlProps {
  /** Whether the dedicated Synced Share GitHub sign-in connection is
   * present -- see `useSyncedShareOwner.ts`. The same identity backs both
   * cloud storage and Synced Share, so this one chip covers both. */
  isGithubConnected: boolean
  /** Cached GitHub username shown as `@login`; `null` until connected. */
  githubLogin: string | null
  /** Drives the chip's small live dot -- true while the active file is
   * currently syncing via Synced Share. */
  isSynced: boolean
  /** Opens the popup "sign in with GitHub" flow -- the same one `ShareModal`
   * and `StorageSettingsModal` use as their own sign-in shortcut. */
  onSignInWithGithub: () => Promise<SyncedShareGithubAuthResult>
  /** The only sign-out action in the app -- stops any active sync, revokes
   * the GitHub grant, clears the token, and falls back to local storage if
   * cloud storage was active (see `App.tsx`'s `handleAccountSignOut`). */
  onSignOut: () => void
}

type SignInStatus =
  | { kind: 'idle' }
  | { kind: 'signing-in' }
  | { kind: 'failed'; error: string }

export function ProfileControl({
  isGithubConnected,
  githubLogin,
  isSynced,
  onSignInWithGithub,
  onSignOut,
}: ProfileControlProps) {
  const [signInStatus, setSignInStatus] = useState<SignInStatus>({
    kind: 'idle',
  })
  const [popoverOpen, setPopoverOpen] = useState(false)

  const handleSignIn = useCallback(() => {
    setSignInStatus({ kind: 'signing-in' })
    void onSignInWithGithub().then((result) => {
      if (result.ok) {
        setSignInStatus({ kind: 'idle' })
      } else if (result.reason === 'error') {
        setSignInStatus({ kind: 'failed', error: result.error })
      } else {
        setSignInStatus({ kind: 'idle' })
      }
    })
  }, [onSignInWithGithub])

  const handleSignOut = useCallback(() => {
    setPopoverOpen(false)
    onSignOut()
  }, [onSignOut])

  if (!isGithubConnected) {
    return (
      <span className="account-chip-signin-wrap">
        <button
          type="button"
          className="preview-export-btn"
          data-testid="account-chip-sign-in"
          disabled={signInStatus.kind === 'signing-in'}
          onClick={handleSignIn}
        >
          <GitHubLogoIcon aria-hidden="true" />
          {signInStatus.kind === 'signing-in' ? 'Signing in…' : 'Sign in'}
        </button>
        {signInStatus.kind === 'failed' && (
          <span
            className="account-chip-signin-error"
            role="alert"
            data-testid="account-chip-signin-error"
          >
            {signInStatus.error}
          </span>
        )}
      </span>
    )
  }

  return (
    <Popover.Root open={popoverOpen} onOpenChange={setPopoverOpen}>
      <Popover.Trigger asChild>
        <button
          type="button"
          className="account-chip"
          data-testid="account-chip"
        >
          <GitHubLogoIcon aria-hidden="true" />
          {`@${githubLogin}`}
          {isSynced && (
            <span
              className="account-chip-live-dot"
              title="Currently syncing"
              data-testid="account-chip-syncing-indicator"
            />
          )}
        </button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          className="account-popover"
          data-testid="account-popover"
          align="end"
          sideOffset={8}
        >
          <div className="account-popover-head">
            <span className="account-popover-login">{`@${githubLogin}`}</span>
            <span className="account-popover-sub">
              {isSynced ? 'Signed in · syncing this file' : 'Signed in'}
            </span>
          </div>
          <a
            className="account-popover-link"
            href={`https://github.com/${githubLogin}`}
            target="_blank"
            rel="noreferrer"
          >
            <ExternalLinkIcon aria-hidden="true" />
            View GitHub profile
          </a>
          <button
            type="button"
            className="account-popover-signout"
            onClick={handleSignOut}
          >
            <ExitIcon aria-hidden="true" />
            Sign out
          </button>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  )
}
