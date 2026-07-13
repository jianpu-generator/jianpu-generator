import { Octokit } from '@octokit/rest'
import * as Dialog from '@radix-ui/react-dialog'
import { useEffect, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import {
  GITHUB_STORAGE_REPO,
  type StorageBackendPreference,
  type StorageBackendTarget,
} from '../hooks/useStorageBackend'
import {
  checkGithubAuthStatus,
  clearStoredGithubAuth,
  connectWithDeviceFlow,
  type GithubDeviceVerification,
  readStoredGithubAuth,
} from '../storage/githubAuth'
import type { StorageBackend } from '../storage/types'
import {
  type ConflictResolution,
  ensureStorageRepo,
  errorBannerMessage,
  isGithubBackend,
  resolveGithubConflict,
} from './storageSettingsModalHelpers'
import {
  bannerStyle,
  bodyStyle,
  buttonStyle,
  contentStyle,
  headerStyle,
  optionRowStyle,
  overlayStyle,
} from './storageSettingsModalStyles'

export {
  type ConflictResolution,
  ensureStorageRepo,
  resolveGithubConflict,
} from './storageSettingsModalHelpers'

export interface StorageSettingsModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  backend: StorageBackend
  isLoadingGithub: boolean
  preference: StorageBackendPreference
  switchBackend: (target: StorageBackendTarget) => Promise<void>
  store: FileStoreState
  setStore: (
    value: FileStoreState | ((prev: FileStoreState) => FileStoreState),
  ) => void
}

/** Public GitHub OAuth App client ID; not a secret (it's visible in every
 * device-flow request), so it's fine to bake in at build time. The actual
 * value sent to GitHub is injected server-side by the Cloudflare proxy (see
 * `cf-oauth-proxy/functions/device/code.ts`) regardless of what's sent here
 * — this is only required because `@octokit/auth-oauth-device` insists on a
 * non-empty `clientId` to construct its requests. */
const GITHUB_OAUTH_CLIENT_ID = import.meta.env.VITE_GITHUB_OAUTH_CLIENT_ID ?? ''

const GITHUB_OAUTH_PROXY_URL = import.meta.env.VITE_GITHUB_OAUTH_PROXY_URL ?? ''

export function StorageSettingsModal({
  open,
  onOpenChange,
  backend,
  isLoadingGithub,
  preference,
  switchBackend,
  store,
  setStore,
}: StorageSettingsModalProps) {
  const [selectedKind, setSelectedKind] = useState<'local' | 'github'>(
    preference.backend,
  )
  const [username, setUsername] = useState<string | null>(
    preference.github?.owner ?? null,
  )
  const [connecting, setConnecting] = useState(false)
  const [verification, setVerification] =
    useState<GithubDeviceVerification | null>(null)
  const [connectError, setConnectError] = useState<string | null>(null)

  useEffect(() => {
    if (!open) return
    setSelectedKind(preference.backend)
    setConnectError(null)
    setVerification(null)
  }, [open, preference])

  useEffect(() => {
    if (!open) return
    if (!readStoredGithubAuth()) {
      setUsername(null)
      return
    }
    let cancelled = false
    checkGithubAuthStatus().then((status) => {
      if (cancelled) return
      if (status.state === 'connected') setUsername(status.username)
      else setUsername(null)
    })
    return () => {
      cancelled = true
    }
  }, [open])

  async function handleConnect() {
    setConnecting(true)
    setConnectError(null)
    setVerification(null)
    try {
      await connectWithDeviceFlow({
        clientId: GITHUB_OAUTH_CLIENT_ID,
        proxyBaseUrl: GITHUB_OAUTH_PROXY_URL,
        scopes: ['repo'],
        onVerification: (v) => setVerification(v),
      })
      const stored = readStoredGithubAuth()
      if (!stored) throw new Error('Connection did not persist a token')
      const octokit = new Octokit({ auth: stored.token })
      const { data: user } = await octokit.rest.users.getAuthenticated()
      await ensureStorageRepo(octokit, user.login)
      setUsername(user.login)
      await switchBackend({ kind: 'github', owner: user.login })
    } catch (error) {
      setConnectError(error instanceof Error ? error.message : String(error))
    } finally {
      setConnecting(false)
      setVerification(null)
    }
  }

  function handleDisconnect() {
    clearStoredGithubAuth()
    setUsername(null)
    void switchBackend({ kind: 'local' })
  }

  async function handleSelectLocal() {
    setSelectedKind('local')
    await switchBackend({ kind: 'local' })
  }

  async function handleSelectGithub(currentUsername: string | null) {
    setSelectedKind('github')
    if (currentUsername) {
      await switchBackend({ kind: 'github', owner: currentUsername })
    }
  }

  const githubBackendError = isGithubBackend(backend)
    ? backend.lastError()
    : null
  const bannerMessage = errorBannerMessage(githubBackendError)
  const conflictPath =
    githubBackendError?.kind === 'conflict' ? githubBackendError.path : null

  async function handleResolveConflict(resolution: ConflictResolution) {
    if (!isGithubBackend(backend)) return
    const nextStore = await resolveGithubConflict(resolution, backend, store)
    setStore(nextStore)
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
            {conflictPath ? (
              <div style={bannerStyle} data-testid="conflict-banner">
                <p style={{ margin: '0 0 6px' }}>
                  "{conflictPath}" changed on GitHub since your last save.
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
              <label
                style={{ display: 'flex', gap: '6px', alignItems: 'center' }}
              >
                <input
                  type="radio"
                  name="storage-backend"
                  checked={selectedKind === 'local'}
                  onChange={handleSelectLocal}
                />
                This browser
              </label>
              <label
                style={{ display: 'flex', gap: '6px', alignItems: 'center' }}
              >
                <input
                  type="radio"
                  name="storage-backend"
                  checked={selectedKind === 'github'}
                  onChange={() => void handleSelectGithub(username)}
                />
                GitHub repository
              </label>
            </div>

            {selectedKind === 'github' ? (
              username ? (
                <div
                  style={{
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '8px',
                  }}
                  data-testid="github-connected"
                >
                  <p style={{ margin: 0 }}>
                    Connected as{' '}
                    <a
                      href={`https://github.com/${username}`}
                      target="_blank"
                      rel="noreferrer"
                    >
                      <strong>@{username}</strong>
                    </a>
                  </p>
                  <p style={{ margin: 0, color: '#666' }}>
                    Storing files in{' '}
                    <a
                      href={`https://github.com/${username}/${GITHUB_STORAGE_REPO}`}
                      target="_blank"
                      rel="noreferrer"
                    >
                      {username}/{GITHUB_STORAGE_REPO}
                    </a>
                    <code>/scores</code>
                  </p>
                  {isLoadingGithub ? (
                    <p
                      style={{
                        margin: 0,
                        display: 'flex',
                        alignItems: 'center',
                        gap: '6px',
                        color: '#666',
                      }}
                      data-testid="github-loading-spinner"
                    >
                      <span
                        className="file-tab-bar-spinner"
                        aria-hidden="true"
                      />
                      Loading files from GitHub…
                    </p>
                  ) : null}
                  <button
                    type="button"
                    style={{ ...buttonStyle, alignSelf: 'flex-start' }}
                    onClick={handleDisconnect}
                  >
                    Disconnect
                  </button>
                </div>
              ) : (
                <div
                  style={{
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '8px',
                  }}
                  data-testid="github-connect"
                >
                  {verification ? (
                    <div data-testid="device-verification">
                      <p style={{ margin: '0 0 4px' }}>
                        Go to{' '}
                        <a
                          href={verification.verification_uri}
                          target="_blank"
                          rel="noreferrer"
                        >
                          {verification.verification_uri}
                        </a>{' '}
                        and enter this code:
                      </p>
                      <p
                        style={{
                          fontSize: '18px',
                          fontWeight: 700,
                          margin: '0 0 4px',
                        }}
                      >
                        {verification.user_code}
                      </p>
                      <p style={{ margin: 0, color: '#666' }}>
                        Waiting for authorization…
                      </p>
                    </div>
                  ) : (
                    <button
                      type="button"
                      style={buttonStyle}
                      onClick={handleConnect}
                      disabled={connecting}
                    >
                      {connecting ? 'Connecting…' : 'Connect GitHub'}
                    </button>
                  )}
                  {connectError ? (
                    <p style={{ color: '#b00020', margin: 0 }}>
                      {connectError}
                    </p>
                  ) : null}
                </div>
              )
            ) : null}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
