import { useEffect, useRef, useState } from 'react'
import { completeSyncedShareGithubSignInFromCallback } from '../storage/syncedShareGithubAuthCallback'

/**
 * Rendered (by `main.tsx`, based on `window.location.pathname`) at
 * `syncedShareGithubCallbackPathname()` -- the page GitHub's popup redirects
 * back to once the user approves or denies the Synced Share sign-in
 * request. Its only job is to run `completeSyncedShareGithubSignInFromCallback`
 * (which exchanges the code, persists the token, and `postMessage`s the
 * outcome to the opener window per `syncedShareGithubAuth.ts`) and then
 * close itself -- the opener's own UI (`useSyncedShareOwner.ts` /
 * `SyncedShareButton`) is what actually reacts to the result.
 *
 * The status text below matters whenever `window.close()` is refused --
 * not just the "opened directly, not as a popup" case, but also the popup
 * flow itself: GitHub's authorization endpoint sends
 * `Cross-Origin-Opener-Policy: same-origin`, which permanently severs this
 * page's `window.opener` once the popup has navigated there, and browsers
 * refuse to let a script close a window that no longer reports an opener.
 * The call below is kept as a best-effort (it still works whenever the
 * browser hasn't severed the opener), but the actual close in that case
 * comes from the *opener*'s side instead, via the `localStorage` relay in
 * `completeSyncedShareGithubSignInFromCallback` /
 * `openSyncedShareGithubSignInPopup` (see
 * `SYNCED_SHARE_GITHUB_AUTH_RELAY_STORAGE_KEY`'s doc comment) -- so this
 * page must never leave the user staring at nothing in the meantime.
 */
export function SyncedShareGithubCallbackPage() {
  const [status, setStatus] = useState<'working' | 'done' | 'failed'>('working')
  const [message, setMessage] = useState<string | null>(null)
  // GitHub authorization codes are single-use: the token exchange must run
  // at most once per popup round-trip. `StrictMode` (see `main.tsx`) mounts
  // this effect, cleans it up, then mounts it again in dev -- without this
  // guard that fires the exchange twice for the same `code`, and GitHub
  // honors only the first request, failing the second with "The code passed
  // is incorrect or expired". A plain effect-scoped `cancelled` flag doesn't
  // prevent this: it only suppresses acting on a stale *result*, not the
  // second *fetch* itself -- this ref persists across the double-mount
  // (StrictMode preserves component state/refs, it only re-runs effects) so
  // the second effect run sees it's already started and skips the call.
  const startedRef = useRef(false)

  useEffect(() => {
    if (startedRef.current) return
    startedRef.current = true
    let cancelled = false
    const host = import.meta.env.VITE_SYNCED_SHARE_HOST ?? ''
    const finish = (
      result:
        | Awaited<
            ReturnType<typeof completeSyncedShareGithubSignInFromCallback>
          >
        | { ok: false; error: string },
    ) => {
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
    }
    void completeSyncedShareGithubSignInFromCallback({ host }).then(
      finish,
      // `completeSyncedShareGithubSignInFromCallback` shouldn't itself
      // reject (every failure path inside it resolves a `{ ok: false }`
      // result instead) -- this `onRejected` is a last-resort backstop, not
      // the primary handling. Without it, any exception that does slip
      // through becomes an unhandled rejection and this page is stuck
      // rendering "Signing in with GitHub…" forever (the state update and
      // `window.close()` above never run) -- exactly the bug this guards.
      (error: unknown) => {
        finish({
          ok: false,
          error: `Unexpected error completing GitHub sign-in: ${error instanceof Error ? error.message : String(error)}`,
        })
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
