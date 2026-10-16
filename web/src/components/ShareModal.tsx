import * as Dialog from '@radix-ui/react-dialog'
import { GitHubLogoIcon, Link2Icon, UpdateIcon } from '@radix-ui/react-icons'
import { useCallback, useEffect, useState } from 'react'
import type { StaticShareLink } from '../hooks/useStaticShareLink'
import type { SyncedShareGithubAuthResult } from '../storage/accountAuthPopup'
import { StopSyncControl } from './StopSyncControl'
import { helpTextStyle, linkFieldStyle, linkRowStyle } from './shareModalStyles'
import {
  backendButtonSelectedStyle,
  backendButtonStyle,
  bannerStyle,
  bodyStyle,
  buttonStyle,
  contentStyle,
  headerStyle,
  optionRowStyle,
  overlayStyle,
  primaryButtonStyle,
  statusLineStyle,
} from './storageSettingsModalStyles'

export interface ShareModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** What is being shared: a single file or a whole folder. */
  kind: 'file' | 'folder'
  /** The file's path, or the folder's own name; shown in titles. */
  name: string
  /** The snapshot link; `null` while it is being built. */
  staticLink: StaticShareLink | null
  /** Tab shown when the dialog opens. A folder's "Sharing settings" opens on
   * the synced tab. */
  initialTab?: Tab
  /** Said above "Start Sync" (a folder whose files already have their own
   * links); from Rust. */
  startWarning?: string | null
  /** Whether the target can have a live link at all -- only a file (or the
   * files of a folder) saved to cloud storage can, since the link points at
   * cloud rows (see `useSyncedShareOwner.ts`). */
  canSync: boolean
  isSynced: boolean
  syncedShareLink: string | null
  /** Whether the dedicated Synced Share GitHub sign-in connection is
   * present -- see `useSyncedShareOwner.ts`. Gates which of the Synced-link
   * tab's states renders. */
  isGithubConnected: boolean
  /** Cached GitHub username for the "Synced as @username" identity row;
   * `null` until connected. */
  githubLogin: string | null
  onStartSync: () => Promise<string | null>
  onStopSync: () => void
  /** Opens the popup "sign in with GitHub" flow. Never itself starts a
   * share -- the user must click "Start Sync" again once connected. */
  onSignInWithGithub: () => Promise<SyncedShareGithubAuthResult>
}

export type Tab = 'static' | 'synced'

type SignInStatus =
  | { kind: 'idle' }
  | { kind: 'signing-in' }
  | { kind: 'done'; login: string }
  | { kind: 'failed'; error: string }

function countLabel(count: number | undefined, noun: string): string {
  const n = count ?? 0
  return `${n} ${noun}${n === 1 ? '' : 's'}`
}

/** Generalizes `ShareButton`'s clipboard/fallback-prompt copy logic to take
 * a URL param, so each tab of `ShareModal` can own its own "Copy" ->
 * "Link copied" flash without one tab's flash leaking into the other. */
function useCopyLabel(defaultLabel: string) {
  const [copied, setCopied] = useState(false)

  const copy = useCallback(async (url: string) => {
    try {
      await navigator.clipboard.writeText(url)
      setCopied(true)
      window.setTimeout(() => setCopied(false), 2000)
    } catch {
      window.prompt('Copy this link to share:', url)
    }
  }, [])

  return { label: copied ? 'Link copied' : defaultLabel, copy }
}

export function ShareModal({
  open,
  onOpenChange,
  kind,
  name,
  staticLink,
  initialTab = 'static',
  startWarning = null,
  canSync,
  isSynced,
  syncedShareLink,
  isGithubConnected,
  githubLogin,
  onStartSync,
  onStopSync,
  onSignInWithGithub,
}: ShareModalProps) {
  const [tab, setTab] = useState<Tab>(initialTab)
  const [signInStatus, setSignInStatus] = useState<SignInStatus>({
    kind: 'idle',
  })

  const staticCopy = useCopyLabel('Copy')
  const syncedCopy = useCopyLabel('Copy')

  useEffect(() => {
    if (!open) return
    setTab(initialTab)
    setSignInStatus({ kind: 'idle' })
  }, [open, initialTab])

  const noun = kind === 'folder' ? 'folder' : 'file'

  const handleSignInWithGithub = useCallback(() => {
    setSignInStatus({ kind: 'signing-in' })
    void onSignInWithGithub().then((result) => {
      if (result.ok) {
        setSignInStatus({ kind: 'done', login: result.login })
      } else if (result.reason === 'error') {
        setSignInStatus({ kind: 'failed', error: result.error })
      } else {
        setSignInStatus({ kind: 'idle' })
      }
    })
  }, [onSignInWithGithub])

  const handleStartSync = useCallback(() => {
    void onStartSync().then((link) => {
      if (link) void syncedCopy.copy(link)
    })
  }, [onStartSync, syncedCopy])

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay style={overlayStyle} />
        <Dialog.Content data-testid="share-modal" style={contentStyle}>
          <div style={headerStyle}>
            <Dialog.Title
              style={{ margin: 0, fontSize: '14px', fontWeight: 600 }}
            >
              {kind === 'folder' ? `Share folder "${name}"` : 'Share'}
            </Dialog.Title>
            <Dialog.Close
              style={{
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                fontSize: '16px',
                color: '#666',
                lineHeight: 1,
                padding: '2px 4px',
              }}
            >
              ×
            </Dialog.Close>
          </div>
          <div style={bodyStyle}>
            <div style={optionRowStyle}>
              <button
                type="button"
                aria-pressed={tab === 'static'}
                aria-label="Static link"
                data-testid="share-modal-static-tab"
                style={
                  tab === 'static'
                    ? backendButtonSelectedStyle
                    : backendButtonStyle
                }
                onClick={() => setTab('static')}
              >
                <Link2Icon width={28} height={28} />
                Static link
              </button>
              <button
                type="button"
                aria-pressed={tab === 'synced'}
                aria-label="Synced link"
                data-testid="share-modal-synced-tab"
                style={
                  tab === 'synced'
                    ? backendButtonSelectedStyle
                    : backendButtonStyle
                }
                onClick={() => setTab('synced')}
              >
                <UpdateIcon width={28} height={28} />
                Synced link
              </button>
            </div>

            {tab === 'static' ? (
              <div
                style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}
              >
                {staticLink?.tooLong ? (
                  <>
                    <div style={bannerStyle} data-testid="share-modal-too-big">
                      This folder has {countLabel(staticLink.fileCount, 'file')}
                      , which is too much for a link to carry reliably. Use a
                      synced link instead.
                    </div>
                    <button
                      type="button"
                      style={primaryButtonStyle}
                      data-testid="share-modal-use-synced-link"
                      onClick={() => setTab('synced')}
                    >
                      Use synced link
                    </button>
                  </>
                ) : (
                  <>
                    <div style={linkRowStyle}>
                      <input
                        type="text"
                        readOnly
                        value={staticLink?.url ?? ''}
                        style={linkFieldStyle}
                        data-testid="share-modal-static-link"
                        onFocus={(event) => event.currentTarget.select()}
                      />
                      <button
                        type="button"
                        style={buttonStyle}
                        data-testid="share-modal-copy-static-link"
                        onClick={() =>
                          void staticCopy.copy(staticLink?.url ?? '')
                        }
                      >
                        {staticCopy.label}
                      </button>
                    </div>
                    <p
                      style={helpTextStyle}
                      data-testid="share-modal-static-note"
                    >
                      {kind === 'folder'
                        ? `A snapshot of ${countLabel(staticLink?.fileCount, 'file')} in ${countLabel(staticLink?.folderCount, 'folder')}. Future edits and new files won't appear here.`
                        : "A snapshot of right now — future edits won't appear here."}
                    </p>
                  </>
                )}
              </div>
            ) : !isGithubConnected ? (
              <div
                style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}
                data-testid="share-modal-signin-state"
              >
                <p style={helpTextStyle}>
                  Sign in with GitHub to share a link that always shows your
                  latest cloud save.
                </p>
                {signInStatus.kind === 'failed' && (
                  <p
                    style={{ margin: 0, color: '#b00020', fontSize: '12px' }}
                    data-testid="share-modal-signin-error"
                  >
                    {signInStatus.error}
                  </p>
                )}
                <button
                  type="button"
                  style={primaryButtonStyle}
                  data-testid="share-modal-sign-in-with-github"
                  disabled={signInStatus.kind === 'signing-in'}
                  onClick={handleSignInWithGithub}
                >
                  <GitHubLogoIcon aria-hidden="true" />{' '}
                  {signInStatus.kind === 'signing-in'
                    ? 'Signing in…'
                    : 'Sign in with GitHub'}
                </button>
              </div>
            ) : !canSync ? (
              <div
                style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}
              >
                <p style={statusLineStyle} data-testid="share-modal-identity">
                  Signed in as <strong>@{githubLogin}</strong>
                </p>
                <p
                  style={helpTextStyle}
                  data-testid="share-modal-synced-cloud-only"
                >
                  Live links are available for {noun}s saved to cloud storage.
                </p>
              </div>
            ) : !isSynced ? (
              <div
                style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}
              >
                <p style={statusLineStyle} data-testid="share-modal-identity">
                  Signed in as <strong>@{githubLogin}</strong>
                </p>
                <p style={helpTextStyle}>
                  {kind === 'folder'
                    ? "Anyone with this link sees the files in this folder as last saved to the cloud, including files you add later. Don't share it publicly."
                    : "Anyone with this link sees this file as last saved to the cloud. Don't share it publicly."}
                </p>
                {startWarning ? (
                  <div
                    style={bannerStyle}
                    data-testid="share-modal-own-links-warning"
                  >
                    {startWarning}
                  </div>
                ) : null}
                <button
                  type="button"
                  style={primaryButtonStyle}
                  data-testid="share-modal-start-sync"
                  onClick={handleStartSync}
                >
                  Start Sync
                </button>
              </div>
            ) : (
              <div
                style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}
              >
                <p style={statusLineStyle} data-testid="share-modal-identity">
                  Synced as <strong>@{githubLogin}</strong>
                </p>
                <div style={linkRowStyle}>
                  <input
                    type="text"
                    readOnly
                    value={syncedShareLink ?? ''}
                    style={linkFieldStyle}
                    data-testid="share-modal-synced-link"
                    onFocus={(event) => event.currentTarget.select()}
                  />
                  <button
                    type="button"
                    style={buttonStyle}
                    data-testid="share-modal-copy-synced-link"
                    onClick={() => {
                      if (syncedShareLink) void syncedCopy.copy(syncedShareLink)
                    }}
                  >
                    {syncedCopy.label}
                  </button>
                </div>
                {kind === 'folder' ? (
                  <p style={helpTextStyle}>
                    Includes every file in this folder, now and added later.
                    Viewers see changes when they reload, and cannot edit.
                  </p>
                ) : null}
                <p style={helpTextStyle}>
                  Stopping ends the link for viewers, but syncing again reuses
                  it.
                </p>
                <StopSyncControl kind={kind} name={name} onStop={onStopSync} />
              </div>
            )}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
