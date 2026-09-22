import * as Dialog from '@radix-ui/react-dialog'
import { GitHubLogoIcon, LaptopIcon, UploadIcon } from '@radix-ui/react-icons'
import { useCallback, useEffect, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import type {
  StorageBackendPreference,
  StorageBackendTarget,
} from '../hooks/useStorageBackend'
import { useAccountAuth } from '../storage/accountAuth'
import {
  openSyncedShareGithubSignInPopup,
  type SyncedShareGithubAuthResult,
} from '../storage/accountAuthPopup'
import type { StorageBackend } from '../storage/types'
import {
  type ConflictResolution,
  errorBannerMessage,
  isCloudBackend,
  resolveCloudConflict,
} from './storageSettingsModalHelpers'
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

export {
  type ConflictResolution,
  resolveCloudConflict,
} from './storageSettingsModalHelpers'

export interface StorageSettingsModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  backend: StorageBackend
  isLoadingCloud: boolean
  preference: StorageBackendPreference
  switchBackend: (target: StorageBackendTarget) => Promise<void>
  store: FileStoreState
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void
  /** Re-syncs `useStorageBackend`'s `saveStatus` state after a conflict
   * resolution mutates the `CloudBackend`'s status directly (via
   * `resolveCloudConflict`'s `forceOverwrite`/`load` calls, which bypass
   * the hook's own `runSave`), so the tab bar's "Saved" badge stops showing
   * the conflict's stale error status once resolved. */
  refreshSaveStatus: (syncedStore?: FileStoreState) => void
}

/** Public "account" GitHub OAuth App client id used for the unified
 * sign-in — the same dedicated, minimally-scoped app (`GET /user` only)
 * `useSyncedShareOwner.ts` uses for Synced Share ownership. Not a secret:
 * it's visible in every authorization request the browser sends. */
const ACCOUNT_GITHUB_OAUTH_CLIENT_ID =
  import.meta.env.VITE_SYNCED_SHARE_GITHUB_OAUTH_CLIENT_ID ?? ''

type SignInStatus =
  | { kind: 'idle' }
  | { kind: 'signing-in' }
  | { kind: 'failed'; error: string }

export function StorageSettingsModal({
  open,
  onOpenChange,
  backend,
  isLoadingCloud,
  preference,
  switchBackend,
  store,
  setStore,
  refreshSaveStatus,
}: StorageSettingsModalProps) {
  const [accountAuth] = useAccountAuth()
  const [selectedKind, setSelectedKind] = useState<'local' | 'cloud'>(
    preference.backend,
  )
  const [signInStatus, setSignInStatus] = useState<SignInStatus>({
    kind: 'idle',
  })

  useEffect(() => {
    if (!open) return
    setSelectedKind(preference.backend)
    setSignInStatus({ kind: 'idle' })
  }, [open, preference])

  const handleSignIn = useCallback(() => {
    setSignInStatus({ kind: 'signing-in' })
    void openSyncedShareGithubSignInPopup({
      clientId: ACCOUNT_GITHUB_OAUTH_CLIENT_ID,
    }).then((result: SyncedShareGithubAuthResult) => {
      if (result.ok) {
        setSignInStatus({ kind: 'idle' })
      } else if (result.reason === 'error') {
        setSignInStatus({ kind: 'failed', error: result.error })
      } else {
        setSignInStatus({ kind: 'idle' })
      }
    })
  }, [])

  async function handleSelectLocal() {
    setSelectedKind('local')
    await switchBackend({ kind: 'local' })
  }

  async function handleSelectCloud() {
    setSelectedKind('cloud')
    if (accountAuth) {
      await switchBackend({ kind: 'cloud' })
    }
  }

  const cloudBackendError = isCloudBackend(backend) ? backend.lastError() : null
  const bannerMessage = errorBannerMessage(cloudBackendError)
  const conflictRevision =
    cloudBackendError?.kind === 'conflict'
      ? cloudBackendError.currentRevision
      : null

  async function handleResolveConflict(resolution: ConflictResolution) {
    if (!isCloudBackend(backend)) return
    const nextStore = await resolveCloudConflict(resolution, backend, store)
    setStore(nextStore)
    refreshSaveStatus(nextStore)
  }

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay style={overlayStyle} />
        <Dialog.Content
          data-testid="storage-settings-modal"
          style={contentStyle}
        >
          <div style={headerStyle}>
            <Dialog.Title
              style={{ margin: 0, fontSize: '14px', fontWeight: 600 }}
            >
              Storage
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
            {conflictRevision !== null ? (
              <div style={bannerStyle} data-testid="conflict-banner">
                <p style={{ margin: '0 0 6px' }}>
                  This file changed in the cloud since your last save.
                </p>
                <div style={{ display: 'flex', gap: '8px' }}>
                  <button
                    type="button"
                    style={buttonStyle}
                    onClick={() => handleResolveConflict('overwrite-mine')}
                  >
                    Overwrite mine
                  </button>
                  <button
                    type="button"
                    style={buttonStyle}
                    onClick={() => handleResolveConflict('discard-mine')}
                  >
                    Discard mine
                  </button>
                </div>
              </div>
            ) : null}

            {bannerMessage ? (
              <div style={bannerStyle} data-testid="status-banner">
                {bannerMessage}
              </div>
            ) : null}

            <div style={optionRowStyle}>
              <button
                type="button"
                aria-pressed={selectedKind === 'local'}
                aria-label="This browser"
                style={
                  selectedKind === 'local'
                    ? backendButtonSelectedStyle
                    : backendButtonStyle
                }
                onClick={handleSelectLocal}
              >
                <LaptopIcon width={28} height={28} />
                This browser
              </button>
              <button
                type="button"
                aria-pressed={selectedKind === 'cloud'}
                aria-label="Cloud storage"
                style={
                  selectedKind === 'cloud'
                    ? backendButtonSelectedStyle
                    : backendButtonStyle
                }
                onClick={() => void handleSelectCloud()}
              >
                <UploadIcon width={28} height={28} />
                Cloud storage
              </button>
            </div>

            {accountAuth ? (
              <div
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '8px',
                }}
                data-testid="account-connected"
              >
                <p style={statusLineStyle}>
                  Cloud storage uses <strong>@{accountAuth.login}</strong>
                </p>
                {selectedKind === 'cloud' && isLoadingCloud ? (
                  <p
                    style={{
                      margin: 0,
                      display: 'flex',
                      alignItems: 'center',
                      gap: '6px',
                      color: '#666',
                    }}
                    data-testid="cloud-loading-spinner"
                  >
                    <span className="file-tab-bar-spinner" aria-hidden="true" />
                    Loading files from the cloud…
                  </p>
                ) : null}
              </div>
            ) : (
              <div
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '8px',
                }}
                data-testid="account-sign-in"
              >
                <p style={{ margin: 0, color: '#666' }}>
                  Sign in with GitHub to enable cloud storage.
                </p>
                <button
                  type="button"
                  style={primaryButtonStyle}
                  onClick={handleSignIn}
                  disabled={signInStatus.kind === 'signing-in'}
                >
                  <GitHubLogoIcon aria-hidden="true" />{' '}
                  {signInStatus.kind === 'signing-in'
                    ? 'Signing in…'
                    : 'Sign in with GitHub'}
                </button>
                {signInStatus.kind === 'failed' ? (
                  <p style={{ color: '#b00020', margin: 0 }}>
                    {signInStatus.error}
                  </p>
                ) : null}
              </div>
            )}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
