import { useEffect, useState } from 'react'
import { completeSyncedShareGithubSignInFromCallback } from '../storage/syncedShareGithubAuth'

/**
 * Rendered (by `main.tsx`, based on `window.location.pathname`) at
 * `SYNCED_SHARE_GITHUB_REDIRECT_PATH` -- the page GitHub's popup redirects
 * back to once the user approves or denies the Synced Share sign-in
 * request. Its only job is to run `completeSyncedShareGithubSignInFromCallback`
 * (which exchanges the code, persists the token, and `postMessage`s the
 * outcome to the opener window per `syncedShareGithubAuth.ts`) and then
 * close itself -- the opener's own UI (`useSyncedShareOwner.ts` /
 * `SyncedShareButton`) is what actually reacts to the result.
 *
 * The status text below only matters if `window.close()` is refused (e.g.
 * a browser policy against a script closing a window it didn't itself
 * open with `window.open` -- shouldn't happen in the popup flow this is
 * built for, but this page must never leave the user staring at nothing).
 */
export function SyncedShareGithubCallbackPage() {
  const [status, setStatus] = useState<'working' | 'done' | 'failed'>('working')
  const [message, setMessage] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    const host = import.meta.env.VITE_SYNCED_SHARE_HOST ?? ''
    void completeSyncedShareGithubSignInFromCallback({ host }).then(
      (result) => {
        if (cancelled) return
        if (result.ok) {
          setStatus('done')
        } else {
          setStatus('failed')
          setMessage(result.error)
        }
        // Only closes itself when opened as a popup via `window.open` --
        // browsers refuse to close a window the script didn't open, so this
        // is a no-op (not an error) if the user navigated here directly.
        window.close()
      },
    )
    return () => {
      cancelled = true
    }
  }, [])

  return (
    <div
      data-testid="synced-share-github-callback-page"
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        height: '100vh',
        fontFamily: 'var(--mono, monospace)',
        fontSize: '13px',
        gap: '8px',
        textAlign: 'center',
        padding: '24px',
      }}
    >
      {status === 'working' && <p>Signing in with GitHub…</p>}
      {status === 'done' && <p>Signed in. You can close this window.</p>}
      {status === 'failed' && (
        <>
          <p>GitHub sign-in failed.</p>
          {message ? <p style={{ color: '#b00020' }}>{message}</p> : null}
          <p>You can close this window.</p>
        </>
      )}
    </div>
  )
}
